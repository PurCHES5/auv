use crate::{
  Layer,
  layers::{Cursor, Outline, Status},
};

/// Converts a primitive or composite visual into renderer-ready layers.
pub trait IntoOverlayLayers {
  fn into_overlay_layers(self) -> Vec<Layer>;
}

impl IntoOverlayLayers for Layer {
  fn into_overlay_layers(self) -> Vec<Layer> {
    vec![self]
  }
}

impl IntoOverlayLayers for Cursor {
  fn into_overlay_layers(self) -> Vec<Layer> {
    vec![self.into()]
  }
}

impl IntoOverlayLayers for Outline {
  fn into_overlay_layers(self) -> Vec<Layer> {
    vec![self.into()]
  }
}

impl IntoOverlayLayers for Status {
  fn into_overlay_layers(self) -> Vec<Layer> {
    vec![self.into()]
  }
}
