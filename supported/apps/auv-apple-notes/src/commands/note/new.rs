use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{DEFAULT_APP_ID, DEFAULT_SETTLE_MS, NoteCommandReport};
use crate::driver::NotesDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteNew {
  pub app_id: String,
  pub settle_ms: u64,
}

impl NoteNew {
  pub fn defaults() -> Self {
    Self {
      app_id: DEFAULT_APP_ID.to_string(),
      settle_ms: DEFAULT_SETTLE_MS,
    }
  }

  pub(crate) fn run(&self, driver: &mut impl NotesDriver) -> Result<NoteCommandReport, String> {
    let mut operation = || {
      let settle = Duration::from_millis(self.settle_ms);
      let actions = vec![
        driver.activate_app(&self.app_id, settle)?,
        driver.create_note(&self.app_id, settle)?,
      ];
      Ok(NoteCommandReport {
        command: "note.new",
        actions,
        verification: None,
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_notes.note.new", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
