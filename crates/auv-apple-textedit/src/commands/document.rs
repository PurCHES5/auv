use std::time::Duration;

use auv_driver::DriverResult;
use serde::{Deserialize, Serialize};

use crate::driver::{TextEditActionResult, TextEditDriver, VerificationOutcome};

mod compare;
mod focus;
mod write;

pub use compare::DocumentCompare;
pub use focus::DocumentFocus;
pub use write::DocumentWrite;

pub const DEFAULT_APP_ID: &str = "com.apple.TextEdit";
pub const DEFAULT_MARKER_TEXT: &str = "AUV_TEXTEDIT_MARKER_2026_05_17";
pub const DEFAULT_FOCUS_QUERY: &str = "First Text View";
pub const DEFAULT_BODY_ROLE: &str = "AXTextArea";
pub const DEFAULT_SETTLE_MS: u64 = 250;

/// App-owned entry point for operating on TextEdit's current document.
///
/// The client owns the driver so library, CLI, and MCP callers can share the
/// same document workflow without exposing AX queries or input timing at the
/// primary interface.
pub struct TextEdit<D> {
  driver: D,
  app_id: String,
}

impl<D> TextEdit<D> {
  pub fn new(driver: D) -> Self {
    Self {
      driver,
      app_id: DEFAULT_APP_ID.to_string(),
    }
  }

  /// Returns a short-lived handle to the document currently resolved by
  /// TextEdit. The handle does not cache AX identity; every operation resolves
  /// the live document body through the existing driver workflow.
  pub fn current_document(&mut self) -> Document<'_, D> {
    Document {
      driver: &mut self.driver,
      app_id: &self.app_id,
    }
  }
}

impl TextEdit<crate::driver::MacosTextEditDriver> {
  pub fn open_local() -> DriverResult<Self> {
    crate::driver::MacosTextEditDriver::open_local().map(Self::new)
  }
}

/// A live, app-owned view of TextEdit's current document.
///
/// Its lifetime prevents concurrent use of the underlying UI driver. Because
/// this first seam deliberately carries no persisted AX identity, UI changes
/// are handled by resolving the current document again on the next operation.
pub struct Document<'app, D> {
  driver: &'app mut D,
  app_id: &'app str,
}

impl<D> Document<'_, D>
where
  D: TextEditDriver,
{
  /// Focuses the current document body using TextEdit-owned defaults.
  pub fn focus(&mut self) -> DriverResult<DocumentCommandReport> {
    let activate = self.driver.activate_app(self.app_id, Duration::from_millis(DEFAULT_SETTLE_MS))?;
    let mut report = DocumentFocus {
      app_id: self.app_id.to_string(),
      query: DEFAULT_FOCUS_QUERY.to_string(),
      candidate: String::new(),
    }
    .run(self.driver)?;
    report.actions.insert(0, activate);
    Ok(report)
  }

  /// Replaces the current document body and verifies the written text.
  pub fn write(&mut self, content: impl Into<String>) -> DriverResult<DocumentCommandReport> {
    let mut command = DocumentWrite::defaults_with_content(content);
    command.app_id = self.app_id.to_string();
    command.run_with_checkpoint(self.driver, || Ok::<_, auv_driver::DriverError>(()))
  }

  /// Compares the current document body using the existing contains semantics.
  pub fn compare(&mut self, expected: impl Into<String>) -> DriverResult<DocumentCommandReport> {
    DocumentCompare {
      app_id: self.app_id.to_string(),
      content: expected.into(),
      role: DEFAULT_BODY_ROLE.to_string(),
    }
    .run(self.driver)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentCommand {
  Write(DocumentWrite),
  Compare(DocumentCompare),
  Focus(DocumentFocus),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCommandReport {
  pub command: &'static str,
  pub actions: Vec<TextEditActionResult>,
  pub verification: Option<VerificationOutcome>,
}
