use serde::{Deserialize, Serialize};

use super::{Cursor, Outline, Status};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Layer {
  Cursor(Cursor),
  Outline(Outline),
  Status(Status),
}

impl From<Cursor> for Layer {
  fn from(value: Cursor) -> Self {
    Self::Cursor(value)
  }
}

impl From<Outline> for Layer {
  fn from(value: Outline) -> Self {
    Self::Outline(value)
  }
}

impl From<Status> for Layer {
  fn from(value: Status) -> Self {
    Self::Status(value)
  }
}
