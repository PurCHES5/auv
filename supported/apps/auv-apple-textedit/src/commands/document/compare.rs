use auv_driver::DriverError;
use serde::{Deserialize, Serialize};

use super::DocumentCommandReport;
use crate::driver::TextEditDriver;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCompare {
  pub app_id: String,
  pub content: String,
  pub role: String,
}

impl DocumentCompare {
  pub(crate) fn run(&self, driver: &mut impl TextEditDriver) -> Result<DocumentCommandReport, DriverError> {
    let mut operation = || {
      let verification = driver.verify_ax_text(&self.app_id, &self.content, &self.role)?;
      Ok(DocumentCommandReport {
        command: "document.compare",
        actions: Vec::new(),
        verification: Some(verification),
      })
    };
    #[cfg(feature = "tracing")]
    return auv_tracing::in_span!("auv.apple_textedit.document.compare", operation);
    #[cfg(not(feature = "tracing"))]
    operation()
  }
}
