use serde::{Deserialize, Serialize};

use super::NoteCommandReport;
use crate::driver::NotesDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteCompare {
  pub app_id: String,
  pub content: String,
  pub role: String,
}

impl NoteCompare {
  pub(crate) fn run(&self, driver: &mut impl NotesDriver) -> Result<NoteCommandReport, String> {
    let mut operation = || {
      let verification = driver.verify_ax_text(&self.app_id, &self.content, &self.role)?;
      Ok(NoteCommandReport {
        command: "note.compare",
        actions: Vec::new(),
        verification: Some(verification),
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_notes.note.compare", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
