use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{IntoOverlayLayers, Layer, LifecycleOptions};

/// Renderer-independent visual content presented over the live desktop.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Overlay {
  layers: Vec<Layer>,
}

impl Overlay {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_layer(mut self, layer: impl IntoOverlayLayers) -> Self {
    self.layers.extend(layer.into_overlay_layers());
    self
  }

  pub fn layers(&self) -> &[Layer] {
    &self.layers
  }

  // TODO(overlay-handles): incremental per-layer update and removal are
  // intentionally deferred because current consumers present one-shot visual
  // evidence; add an opaque presentation handle when a runtime consumer needs
  // in-process updates without exposing adapter-owned layer identity.
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
  #[default]
  EaseInOutExpo,
}

/// Animation policy used by a platform renderer when moving overlay layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionOptions {
  duration: Duration,
  easing: Easing,
}

impl MotionOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_duration(mut self, duration: Duration) -> Self {
    self.duration = duration;
    self
  }

  pub fn with_easing(mut self, easing: Easing) -> Self {
    self.easing = easing;
    self
  }

  pub fn duration(self) -> Duration {
    self.duration
  }

  pub fn easing(self) -> Easing {
    self.easing
  }
}

impl Default for MotionOptions {
  fn default() -> Self {
    Self {
      duration: Duration::from_millis(320),
      easing: Easing::EaseInOutExpo,
    }
  }
}

/// Motion and lifecycle policy for showing one overlay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowOptions {
  motion: MotionOptions,
  lifecycle: LifecycleOptions,
}

impl ShowOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_motion_ease(mut self, duration: Duration, easing: Easing) -> Self {
    self.motion = MotionOptions::new().with_duration(duration).with_easing(easing);
    self
  }

  pub fn with_auto_removal_after(mut self, duration: Duration) -> Self {
    self.lifecycle = LifecycleOptions::new().with_auto_removal_after(duration);
    self
  }

  pub fn with_motion_options(mut self, options: MotionOptions) -> Self {
    self.motion = options;
    self
  }

  pub fn with_lifecycle_options(mut self, options: LifecycleOptions) -> Self {
    self.lifecycle = options;
    self
  }

  pub fn motion(self) -> MotionOptions {
    self.motion
  }

  pub fn lifecycle(self) -> LifecycleOptions {
    self.lifecycle
  }
}
