use std::time::Duration;

use crate::commands::note::{DEFAULT_APP_ID, DEFAULT_BODY_ROLE, DEFAULT_FOCUS_QUERY, DEFAULT_SETTLE_MS, NoteCompare, NoteFocus, NoteWrite};
use crate::driver::{MacosNotesDriver, NotesDriver};

/// App-owned entrypoint for common Apple Notes operations.
///
/// The app keeps driver details and Notes-specific selectors behind this
/// interface. `current_note` identifies the note selected in the Notes window;
/// it does not cache a row, coordinate, or cross-run identity.
pub struct NotesApp<D = MacosNotesDriver> {
  app_id: String,
  driver: D,
}

impl NotesApp<MacosNotesDriver> {
  pub fn open_local() -> Result<Self, String> {
    Ok(Self::from_driver(MacosNotesDriver::open_local()?))
  }
}

impl<D> NotesApp<D> {
  /// Builds an app around an existing driver, primarily for embedding and tests.
  pub fn from_driver(driver: D) -> Self {
    Self {
      app_id: DEFAULT_APP_ID.to_string(),
      driver,
    }
  }

  /// Returns a handle to the note currently selected in the Notes window.
  ///
  /// The handle borrows the app so operations cannot accidentally race through
  /// two mutable views of the same application session.
  pub fn current_note(&mut self) -> Note<'_, D> {
    // TODO(notes-folder-model): This slice intentionally models only the
    // selected note because typed sidebar/folder discovery is not available.
    // Add Folder only in an owner-approved typed read/selection slice.
    Note {
      app_id: &self.app_id,
      driver: &mut self.driver,
    }
  }
}

/// A handle to the note currently selected in Apple Notes.
pub struct Note<'a, D> {
  app_id: &'a str,
  driver: &'a mut D,
}

impl<D: NotesDriver> Note<'_, D> {
  /// Activates Notes and focuses the current note body.
  pub fn focus(&mut self) -> Result<(), String> {
    self.driver.activate_app(self.app_id, Duration::from_millis(DEFAULT_SETTLE_MS))?;
    NoteFocus {
      app_id: self.app_id.to_string(),
      query: DEFAULT_FOCUS_QUERY.to_string(),
      candidate: String::new(),
    }
    .run(self.driver)?;
    Ok(())
  }

  /// Writes at the current insertion point in the selected note.
  ///
  /// App activation, body focus, clipboard preservation, and input settling are
  /// Notes implementation details hidden from the caller.
  pub fn write(&mut self, content: impl Into<String>) -> Result<(), String> {
    let command = NoteWrite {
      app_id: self.app_id.to_string(),
      content: content.into(),
      new_note: false,
      replace: false,
      verify: false,
      focus_query: DEFAULT_FOCUS_QUERY.to_string(),
      focus_candidate: String::new(),
      compare_role: DEFAULT_BODY_ROLE.to_string(),
      activate_settle_ms: DEFAULT_SETTLE_MS,
      create_settle_ms: DEFAULT_SETTLE_MS,
      input_settle_ms: DEFAULT_SETTLE_MS,
    };
    command.run(self.driver)?;
    Ok(())
  }

  /// Checks whether the selected note body contains `expected`.
  pub fn compare(&mut self, expected: impl Into<String>) -> Result<bool, String> {
    self.driver.activate_app(self.app_id, Duration::from_millis(DEFAULT_SETTLE_MS))?;
    let expected = expected.into();
    let report = NoteCompare {
      app_id: self.app_id.to_string(),
      content: expected.clone(),
      role: DEFAULT_BODY_ROLE.to_string(),
    }
    .run(self.driver)?;
    let verification = report.verification.ok_or_else(|| "Notes compare completed without a verification value".to_string())?;
    Ok(verification.matched_text.contains(&expected))
  }
}
