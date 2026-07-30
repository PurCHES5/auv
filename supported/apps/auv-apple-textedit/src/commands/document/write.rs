use std::time::Duration;

use auv_driver::DriverError;
use serde::{Deserialize, Serialize};

use super::{DEFAULT_APP_ID, DEFAULT_BODY_ROLE, DEFAULT_FOCUS_QUERY, DEFAULT_SETTLE_MS, DocumentCommandReport};
use crate::driver::TextEditDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentWrite {
  pub app_id: String,
  pub content: String,
  pub replace: bool,
  pub verify: bool,
  pub focus_query: String,
  pub focus_candidate: String,
  pub compare_role: String,
  pub activate_settle_ms: u64,
  pub input_settle_ms: u64,
}

impl DocumentWrite {
  pub fn defaults_with_content(content: impl Into<String>) -> Self {
    Self {
      app_id: DEFAULT_APP_ID.to_string(),
      content: content.into(),
      replace: true,
      verify: true,
      focus_query: DEFAULT_FOCUS_QUERY.to_string(),
      focus_candidate: String::new(),
      compare_role: DEFAULT_BODY_ROLE.to_string(),
      activate_settle_ms: DEFAULT_SETTLE_MS,
      input_settle_ms: DEFAULT_SETTLE_MS,
    }
  }

  /// Runs document writing while checking a caller-owned lifecycle boundary
  /// immediately before each UI-facing driver phase.
  pub fn run_with_checkpoint<E>(
    &self,
    driver: &mut impl TextEditDriver,
    mut checkpoint: impl FnMut() -> Result<(), E>,
  ) -> Result<DocumentCommandReport, E>
  where
    E: From<DriverError>,
  {
    let mut operation = || {
      checkpoint()?;
      let mut actions = vec![driver.activate_app(&self.app_id, Duration::from_millis(self.activate_settle_ms))?];
      checkpoint()?;
      actions.push(driver.focus_text_input(&self.app_id, &self.focus_query, &self.focus_candidate)?);
      checkpoint()?;
      actions.push(driver.paste_text_preserve_clipboard(
        &self.app_id,
        &self.content,
        self.replace,
        Duration::from_millis(self.input_settle_ms),
      )?);
      let verification = if self.verify {
        checkpoint()?;
        Some(driver.verify_ax_text(&self.app_id, &self.content, &self.compare_role)?)
      } else {
        None
      };
      Ok(DocumentCommandReport {
        command: "document.write",
        actions,
        verification,
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_textedit.document.write", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
