use auv_driver::DriverError;
use serde::{Deserialize, Serialize};

use super::DocumentCommandReport;
use crate::driver::TextEditDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentFocus {
  pub app_id: String,
  pub query: String,
  pub candidate: String,
}

impl DocumentFocus {
  pub(crate) fn run(&self, driver: &mut impl TextEditDriver) -> Result<DocumentCommandReport, DriverError> {
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_textedit.document.focus", || {
      let action = driver.focus_text_input(&self.app_id, &self.query, &self.candidate)?;
      Ok(DocumentCommandReport {
        command: "document.focus",
        actions: vec![action],
        verification: None,
      })
    });
    #[cfg(not(feature = "tracing"))]
    {
      let action = driver.focus_text_input(&self.app_id, &self.query, &self.candidate)?;
      Ok(DocumentCommandReport {
        command: "document.focus",
        actions: vec![action],
        verification: None,
      })
    }
  }
}
