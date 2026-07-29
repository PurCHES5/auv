//! Platform-selecting overlay facade.
//!
//! Enable `macos` to route [`show`] and [`remove`] through the AppKit adapter.
//! Renderer-independent types are re-exported from `auv-driver-overlay-common`.

mod error;

pub use auv_driver_overlay_common::*;
pub use error::{OverlayError, OverlayResult};

/// Shows an overlay through the enabled platform adapter.
pub fn show(overlay: &Overlay, options: ShowOptions) -> OverlayResult<()> {
  #[cfg(all(target_os = "macos", feature = "macos"))]
  {
    return auv_driver_overlay_macos::render(overlay, options).map_err(OverlayError::backend);
  }

  #[cfg(not(all(target_os = "macos", feature = "macos")))]
  {
    let _ = (overlay, options);
    Err(OverlayError::Unavailable {
      reason: "no overlay platform adapter is enabled for this target".to_string(),
    })
  }
}

/// Removes all layers owned by the enabled platform adapter.
pub fn remove() -> OverlayResult<()> {
  #[cfg(all(target_os = "macos", feature = "macos"))]
  {
    return auv_driver_overlay_macos::remove().map_err(OverlayError::backend);
  }

  #[cfg(not(all(target_os = "macos", feature = "macos")))]
  {
    Err(OverlayError::Unavailable {
      reason: "no overlay platform adapter is enabled for this target".to_string(),
    })
  }
}
