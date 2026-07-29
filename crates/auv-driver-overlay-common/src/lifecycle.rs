use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Removal policy for overlay layers after they have been shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Removal {
  Manual,
  AutoAfter(Duration),
}

/// Lifecycle policy applied after a platform renderer has rendered an overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleOptions {
  removal: Removal,
}

impl LifecycleOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn manual() -> Self {
    Self {
      removal: Removal::Manual,
    }
  }

  pub fn with_auto_removal_after(mut self, duration: Duration) -> Self {
    self.removal = Removal::AutoAfter(duration);
    self
  }

  pub fn removal(self) -> Removal {
    self.removal
  }
}

impl Default for LifecycleOptions {
  fn default() -> Self {
    Self {
      removal: Removal::AutoAfter(Duration::from_millis(180)),
    }
  }
}
