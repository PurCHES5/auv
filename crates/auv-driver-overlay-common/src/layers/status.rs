use auv_driver_common::ScreenPoint;
use serde::{Deserialize, Serialize};

use crate::style::StatusStyle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
  point: ScreenPoint,
  text: String,
  style: StatusStyle,
}

impl Status {
  pub fn new(point: ScreenPoint, text: impl Into<String>) -> Self {
    Self {
      point,
      text: text.into(),
      style: StatusStyle::default(),
    }
  }

  pub fn with_style(mut self, style: StatusStyle) -> Self {
    self.style = style;
    self
  }

  pub fn point(&self) -> ScreenPoint {
    self.point
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn style(&self) -> StatusStyle {
    self.style
  }
}
