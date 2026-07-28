use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::NoteCommandReport;
use crate::driver::NotesDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteWrite {
  pub app_id: String,
  pub content: String,
  pub new_note: bool,
  pub replace: bool,
  pub verify: bool,
  pub focus_query: String,
  pub focus_candidate: String,
  pub compare_role: String,
  pub activate_settle_ms: u64,
  pub create_settle_ms: u64,
  pub input_settle_ms: u64,
}

impl NoteWrite {
  pub(crate) fn run(&self, driver: &mut impl NotesDriver) -> Result<NoteCommandReport, String> {
    let mut operation = || {
      let mut actions = vec![driver.activate_app(&self.app_id, Duration::from_millis(self.activate_settle_ms))?];
      if self.new_note {
        actions.push(driver.create_note(&self.app_id, Duration::from_millis(self.create_settle_ms))?);
      }
      actions.push(driver.focus_note_body(&self.app_id, &self.focus_query, &self.focus_candidate)?);
      actions.push(driver.paste_text_preserve_clipboard(
        &self.app_id,
        &self.content,
        self.replace,
        Duration::from_millis(self.input_settle_ms),
      )?);
      let verification = if self.verify {
        Some(driver.verify_ax_text(&self.app_id, &self.content, &self.compare_role)?)
      } else {
        None
      };
      Ok(NoteCommandReport {
        command: "note.write",
        actions,
        verification,
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_notes.note.write", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
