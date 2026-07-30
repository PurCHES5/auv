use std::sync::{Arc, Mutex};
use std::time::Duration;

use auv_apple_textedit::{DriverResult, MatchedAxNode, TextEdit, TextEditAction, TextEditActionResult, TextEditDriver, VerificationOutcome};

#[derive(Debug, PartialEq, Eq)]
enum Call {
  Activate {
    app_id: String,
    settle: Duration,
  },
  Focus {
    app_id: String,
    query: String,
    candidate: String,
  },
  Paste {
    app_id: String,
    text: String,
    replace: bool,
    settle: Duration,
  },
  Verify {
    app_id: String,
    text: String,
    role: String,
  },
}

#[derive(Default)]
struct DriverState {
  calls: Vec<Call>,
}

struct RecordingDriver {
  state: Arc<Mutex<DriverState>>,
}

impl RecordingDriver {
  fn record(&self, call: Call) {
    self.state.lock().expect("driver state should not be poisoned").calls.push(call);
  }
}

impl TextEditDriver for RecordingDriver {
  fn activate_app(&mut self, app_id: &str, settle: Duration) -> DriverResult<TextEditActionResult> {
    self.record(Call::Activate {
      app_id: app_id.to_string(),
      settle,
    });
    Ok(action(TextEditAction::Activate))
  }

  fn focus_text_input(&mut self, app_id: &str, query: &str, candidate: &str) -> DriverResult<TextEditActionResult> {
    self.record(Call::Focus {
      app_id: app_id.to_string(),
      query: query.to_string(),
      candidate: candidate.to_string(),
    });
    Ok(action(TextEditAction::FocusTextInput))
  }

  fn paste_text_preserve_clipboard(
    &mut self,
    app_id: &str,
    text: &str,
    replace_existing: bool,
    settle: Duration,
  ) -> DriverResult<TextEditActionResult> {
    self.record(Call::Paste {
      app_id: app_id.to_string(),
      text: text.to_string(),
      replace: replace_existing,
      settle,
    });
    Ok(action(TextEditAction::PasteText))
  }

  fn verify_ax_text(&mut self, app_id: &str, target_text: &str, target_role: &str) -> DriverResult<VerificationOutcome> {
    self.record(Call::Verify {
      app_id: app_id.to_string(),
      text: target_text.to_string(),
      role: target_role.to_string(),
    });
    Ok(VerificationOutcome {
      matched_role: target_role.to_string(),
      matched_text: target_text.to_string(),
      artifact_count: 1,
      semantic_matched: true,
      matched_node: Some(MatchedAxNode {
        path: "0/1".to_string(),
        process_id: 42,
      }),
    })
  }
}

fn action(action: TextEditAction) -> TextEditActionResult {
  TextEditActionResult {
    action,
    input_action_result: None,
  }
}

fn app() -> (TextEdit<RecordingDriver>, Arc<Mutex<DriverState>>) {
  let state = Arc::new(Mutex::new(DriverState::default()));
  let app = TextEdit::new(RecordingDriver {
    state: state.clone(),
  });
  (app, state)
}

#[test]
fn write_current_document_uses_textedit_owned_defaults() {
  let (mut app, state) = app();

  let report = app.current_document().write("hello").expect("write should succeed");

  assert_eq!(report.command, "document.write");
  assert_eq!(report.actions.len(), 3);
  assert!(report.verification.expect("write should verify").semantic_matched);
  assert_eq!(
    state.lock().unwrap().calls,
    [
      Call::Activate {
        app_id: "com.apple.TextEdit".to_string(),
        settle: Duration::from_millis(250),
      },
      Call::Focus {
        app_id: "com.apple.TextEdit".to_string(),
        query: "First Text View".to_string(),
        candidate: String::new(),
      },
      Call::Paste {
        app_id: "com.apple.TextEdit".to_string(),
        text: "hello".to_string(),
        replace: true,
        settle: Duration::from_millis(250),
      },
      Call::Verify {
        app_id: "com.apple.TextEdit".to_string(),
        text: "hello".to_string(),
        role: "AXTextArea".to_string(),
      },
    ]
  );
}

#[test]
fn focus_and_compare_current_document_use_the_owned_workflows() {
  let (mut app, state) = app();

  let focus = app.current_document().focus().expect("focus should succeed");
  let compare = app.current_document().compare("needle").expect("compare should succeed");

  assert_eq!(focus.command, "document.focus");
  assert_eq!(compare.command, "document.compare");
  assert_eq!(
    state.lock().unwrap().calls,
    [
      Call::Activate {
        app_id: "com.apple.TextEdit".to_string(),
        settle: Duration::from_millis(250),
      },
      Call::Focus {
        app_id: "com.apple.TextEdit".to_string(),
        query: "First Text View".to_string(),
        candidate: String::new(),
      },
      Call::Verify {
        app_id: "com.apple.TextEdit".to_string(),
        text: "needle".to_string(),
        role: "AXTextArea".to_string(),
      },
    ]
  );
}
