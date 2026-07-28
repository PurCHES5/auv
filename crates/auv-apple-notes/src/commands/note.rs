use serde::{Deserialize, Serialize};

use crate::driver::{NoteActionResult, VerificationOutcome};

mod compare;
mod focus;
mod new;
mod write;

pub use compare::NoteCompare;
pub use focus::NoteFocus;
pub use new::NoteNew;
pub use write::NoteWrite;

pub const DEFAULT_APP_ID: &str = "com.apple.Notes";
pub const DEFAULT_NOTE_TEXT: &str = "AUV_NOTE_MARKER_2026_05_21_V2";
pub const DEFAULT_FOCUS_QUERY: &str = "Note Body Text View";
pub const DEFAULT_BODY_ROLE: &str = "AXTextArea";
pub const DEFAULT_SETTLE_MS: u64 = 250;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteCommand {
  New(NoteNew),
  Write(NoteWrite),
  Compare(NoteCompare),
  Focus(NoteFocus),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteCommandReport {
  pub command: &'static str,
  pub actions: Vec<NoteActionResult>,
  pub verification: Option<VerificationOutcome>,
}
