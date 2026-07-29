use auv_driver_common::Rect;

use crate::{IntoOverlayLayers, Layer, layers::Outline, style::OutlineStyle};

/// A highlighted display or window capture boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureFrame {
  rect: Rect,
  label: Option<String>,
  label_visible: bool,
  style: OutlineStyle,
}

impl CaptureFrame {
  pub fn new(rect: Rect) -> Self {
    Self {
      rect,
      label: None,
      label_visible: false,
      style: OutlineStyle::capture(),
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
}

impl IntoOverlayLayers for CaptureFrame {
  fn into_overlay_layers(self) -> Vec<Layer> {
    let mut outline = Outline::new(self.rect).with_style(self.style);
    if let Some(label) = self.label {
      outline = outline.with_label(label);
    }
    if self.label_visible {
      outline = outline.with_label_visible();
    }
    vec![outline.into()]
  }
}
