use serde::{Deserialize, Serialize};

use super::NoteCommandReport;
use crate::driver::NotesDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteFocus {
  pub app_id: String,
  pub query: String,
  pub candidate: String,
}

impl NoteFocus {
  pub(crate) fn run(&self, driver: &mut impl NotesDriver) -> Result<NoteCommandReport, String> {
    let mut operation = || {
      let action = driver.focus_note_body(&self.app_id, &self.query, &self.candidate)?;
      Ok(NoteCommandReport {
        command: "note.focus",
        actions: vec![action],
        verification: None,
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_notes.note.focus", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
