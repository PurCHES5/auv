use auv_driver_common::Rect;
use serde::{Deserialize, Serialize};

use crate::style::OutlineStyle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outline {
  rect: Rect,
  label: Option<String>,
  label_visible: bool,
  style: OutlineStyle,
}

impl Outline {
  pub fn new(rect: Rect) -> Self {
    Self {
      rect,
      label: None,
      label_visible: false,
      style: OutlineStyle::default(),
    }
  }

  pub fn with_label(mut self, label: impl Into<String>) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn with_label_visible(mut self) -> Self {
    self.label_visible = true;
    self
  }

  pub fn with_style(mut self, style: OutlineStyle) -> Self {
    self.style = style;
    self
  }

  pub fn rect(&self) -> Rect {
    self.rect
  }

  pub fn label(&self) -> Option<&str> {
    self.label.as_deref()
  }

  pub fn label_visible(&self) -> bool {
    self.label_visible
  }

  pub fn style(&self) -> OutlineStyle {
    self.style
  }
}
