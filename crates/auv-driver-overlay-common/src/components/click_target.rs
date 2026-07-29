use auv_driver_common::{Rect, ScreenPoint};

use crate::{
  IntoOverlayLayers, Layer,
  layers::{Cursor, Outline, Status},
  style::{CursorStyle, OutlineStyle, StatusStyle},
};

/// Visual evidence for one click target: optional bounds, cursor, and status.
#[derive(Clone, Debug, PartialEq)]
pub struct ClickTarget {
  point: ScreenPoint,
  outline: Option<Rect>,
  outline_style: OutlineStyle,
  outline_label: Option<String>,
  outline_label_visible: bool,
  cursor_label: Option<String>,
  cursor_label_visible: bool,
  status: Option<(String, ScreenPoint)>,
  status_style: StatusStyle,
  cursor_style: CursorStyle,
}

impl ClickTarget {
  pub fn new(point: ScreenPoint) -> Self {
    Self {
      point,
      outline: None,
      outline_style: OutlineStyle::selected(),
      outline_label: None,
      outline_label_visible: false,
      cursor_label: None,
      cursor_label_visible: false,
      status: None,
      status_style: StatusStyle::action(),
      cursor_style: CursorStyle::auv(),
    }
  }

  pub fn with_outline(mut self, rect: Rect) -> Self {
    self.outline = Some(rect);
    self
  }

  pub fn with_outline_style(mut self, style: OutlineStyle) -> Self {
    self.outline_style = style;
    self
  }

  pub fn with_outline_label(mut self, label: impl Into<String>) -> Self {
    self.outline_label = Some(label.into());
    self
  }

  pub fn with_outline_label_visible(mut self) -> Self {
    self.outline_label_visible = true;
    self
  }

  pub fn with_cursor_label(mut self, label: impl Into<String>) -> Self {
    self.cursor_label = Some(label.into());
    self
  }

  pub fn with_cursor_label_visible(mut self) -> Self {
    self.cursor_label_visible = true;
    self
  }

  pub fn with_status(mut self, text: impl Into<String>) -> Self {
    self.status = Some((text.into(), offset(self.point, 28.0, 28.0)));
    self
  }

  pub fn with_status_at(mut self, point: ScreenPoint, text: impl Into<String>) -> Self {
    self.status = Some((text.into(), point));
    self
  }

  pub fn with_status_style(mut self, style: StatusStyle) -> Self {
    self.status_style = style;
    self
  }

  pub fn with_cursor_style(mut self, style: CursorStyle) -> Self {
    self.cursor_style = style;
    self
  }
}

impl IntoOverlayLayers for ClickTarget {
  fn into_overlay_layers(self) -> Vec<Layer> {
    let mut layers = Vec::with_capacity(3);
    if let Some(rect) = self.outline {
      let mut outline = Outline::new(rect).with_style(self.outline_style);
      if let Some(label) = self.outline_label {
        outline = outline.with_label(label);
      }
      if self.outline_label_visible {
        outline = outline.with_label_visible();
      }
      layers.push(outline.into());
    }

    let mut cursor = Cursor::new(self.point).with_style(self.cursor_style);
    if let Some(label) = self.cursor_label {
      cursor = cursor.with_label(label);
    }
    if self.cursor_label_visible {
      cursor = cursor.with_label_visible();
    }
    layers.push(cursor.into());

    if let Some((text, point)) = self.status {
      layers.push(Status::new(point, text).with_style(self.status_style).into());
    }
    layers
  }
}

fn offset(point: ScreenPoint, x: f64, y: f64) -> ScreenPoint {
  ScreenPoint::new(point.point().x + x, point.point().y + y)
}
