use std::sync::{Arc, Mutex};
use std::time::Duration;

use auv_apple_notes::{NoteAction, NoteActionResult, NotesApp, NotesDriver, VerificationOutcome};

#[derive(Default)]
struct DriverState {
  calls: Vec<String>,
  matched_text: String,
}

struct RecordingDriver {
  state: Arc<Mutex<DriverState>>,
}

impl RecordingDriver {
  fn action(action: NoteAction) -> NoteActionResult {
    NoteActionResult {
      action,
      input_action_result: None,
    }
  }

  fn record(&self, call: String) {
    self.state.lock().expect("driver state should not be poisoned").calls.push(call);
  }
}

impl NotesDriver for RecordingDriver {
  fn activate_app(&mut self, app_id: &str, settle: Duration) -> Result<NoteActionResult, String> {
    self.record(format!("activate:{app_id}:{}", settle.as_millis()));
    Ok(Self::action(NoteAction::Activate))
  }

  fn create_note(&mut self, app_id: &str, settle: Duration) -> Result<NoteActionResult, String> {
    self.record(format!("create:{app_id}:{}", settle.as_millis()));
    Ok(Self::action(NoteAction::Create))
  }

  fn focus_note_body(&mut self, app_id: &str, query: &str, candidate: &str) -> Result<NoteActionResult, String> {
    self.record(format!("focus:{app_id}:{query}:{candidate}"));
    Ok(Self::action(NoteAction::FocusBody))
  }

  fn paste_text_preserve_clipboard(
    &mut self,
    app_id: &str,
    text: &str,
    replace_existing: bool,
    settle: Duration,
  ) -> Result<NoteActionResult, String> {
    self.record(format!("paste:{app_id}:{text}:{replace_existing}:{}", settle.as_millis()));
    Ok(Self::action(NoteAction::PasteText))
  }

  fn verify_ax_text(&mut self, app_id: &str, target_text: &str, target_role: &str) -> Result<VerificationOutcome, String> {
    self.record(format!("compare:{app_id}:{target_text}:{target_role}"));
    Ok(VerificationOutcome {
      matched_role: target_role.to_string(),
      matched_text: self.state.lock().expect("driver state should not be poisoned").matched_text.clone(),
      artifact_count: 0,
    })
  }
}

fn app_with_text(matched_text: &str) -> (NotesApp<RecordingDriver>, Arc<Mutex<DriverState>>) {
  let state = Arc::new(Mutex::new(DriverState {
    calls: Vec::new(),
    matched_text: matched_text.to_string(),
  }));
  let app = NotesApp::from_driver(RecordingDriver {
    state: state.clone(),
  });
  (app, state)
}

#[test]
fn write_selected_note_uses_the_notes_owned_input_sequence() {
  let (mut app, state) = app_with_text("");

  app.current_note().write("hello").expect("write should succeed");

  assert_eq!(
    state.lock().unwrap().calls,
    [
      "activate:com.apple.Notes:250",
      "focus:com.apple.Notes:Note Body Text View:",
      "paste:com.apple.Notes:hello:false:250",
    ]
  );
}

#[test]
fn focus_selected_note_activates_notes_before_focusing() {
  let (mut app, state) = app_with_text("");

  app.current_note().focus().expect("focus should succeed");

  assert_eq!(
    state.lock().unwrap().calls,
    [
      "activate:com.apple.Notes:250",
      "focus:com.apple.Notes:Note Body Text View:"
    ]
  );
}

#[test]
fn compare_selected_note_reports_match_and_mismatch() {
  let (mut matching_app, matching_state) = app_with_text("title\nexpected body text");
  let (mut other_app, _) = app_with_text("different text");

  assert!(matching_app.current_note().compare("expected body").expect("compare should succeed"));
  assert!(!other_app.current_note().compare("expected").expect("compare should complete"));
  assert_eq!(
    matching_state.lock().unwrap().calls,
    [
      "activate:com.apple.Notes:250",
      "compare:com.apple.Notes:expected body:AXTextArea"
    ]
  );
}
