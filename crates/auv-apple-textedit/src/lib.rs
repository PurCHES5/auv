pub mod cli;
pub mod commands;
pub mod driver;

pub use auv_driver::DriverResult;
pub use commands::document::{
  DEFAULT_APP_ID, DEFAULT_BODY_ROLE, DEFAULT_FOCUS_QUERY, DEFAULT_MARKER_TEXT, DEFAULT_SETTLE_MS, Document, DocumentCommand,
  DocumentCommandReport, DocumentCompare, DocumentFocus, DocumentWrite, TextEdit,
};
pub use driver::{MacosTextEditDriver, MatchedAxNode, TextEditAction, TextEditActionResult, TextEditDriver, VerificationOutcome};
