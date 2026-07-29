use crate::AuvResult;
use auv_driver_overlay_common::{Easing, Layer, Overlay, Removal, ShowOptions, layers::CursorImage};

/// Renders an ordered overlay through the native AppKit adapter.
///
/// Layer identity is adapter-owned. A stable kind-relative index lets the
/// native controller inherit cursor positions across consecutive renders
/// without exposing implementation identifiers to callers.
pub fn render(overlay: &Overlay, options: ShowOptions) -> AuvResult<()> {
  let motion_duration_ms = duration_millis(options.motion().duration())?;
  // NOTICE: the Swift renderer currently implements ease-in-out-expo as the
  // sole shared easing contract. Extend the native boundary when Easing gains
  // another variant.
  match options.motion().easing() {
    Easing::EaseInOutExpo => {}
  }
  let mut cursor_index = 0usize;
  let mut outline_index = 0usize;
  let mut status_index = 0usize;

  for layer in overlay.layers() {
    match layer {
      Layer::Cursor(cursor) => {
        let id = internal_id("cursor", cursor_index);
        cursor_index += 1;
        match cursor.image() {
          CursorImage::BuiltIn { variant } => {
            crate::native::overlay::move_cursor(&id, cursor, variant.as_str(), motion_duration_ms)?;
          }
          CursorImage::Svg { source } => {
            if source.len() > 256 * 1024 {
              return Err("overlay cursor SVG exceeds the 256 KiB runtime limit".to_string());
            }
            crate::native::overlay::move_svg_cursor(&id, cursor, source, motion_duration_ms)?;
          }
        }
      }
      Layer::Outline(outline) => {
        let id = internal_id("outline", outline_index);
        outline_index += 1;
        crate::native::overlay::show_outline(&id, outline, motion_duration_ms)?;
      }
      Layer::Status(status) => {
        let id = internal_id("status", status_index);
        status_index += 1;
        crate::native::overlay::show_status(&id, status, motion_duration_ms)?;
      }
    }
  }

  match options.lifecycle().removal() {
    Removal::Manual => {}
    Removal::AutoAfter(duration) => {
      let duration_ms = duration_millis(duration)?;
      if duration_ms > 0 {
        crate::native::overlay::pump_events(duration_ms)?;
      }
      remove()?;
    }
  }

  Ok(())
}

pub fn remove() -> AuvResult<()> {
  crate::native::overlay::hide_all()
}

fn duration_millis(duration: std::time::Duration) -> AuvResult<u64> {
  u64::try_from(duration.as_millis()).map_err(|_| "overlay duration exceeds the native renderer limit".to_string())
}

fn internal_id(kind: &str, index: usize) -> String {
  format!("auv-{kind}-{index}")
}

#[cfg(test)]
mod tests {
  use super::internal_id;

  #[test]
  fn native_layer_identity_is_private_and_deterministic() {
    assert_eq!(internal_id("cursor", 0), "auv-cursor-0");
    assert_eq!(internal_id("outline", 2), "auv-outline-2");
  }
}
