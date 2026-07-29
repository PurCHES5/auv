use std::time::Duration;

use super::{
  Easing, IntoOverlayLayers, Layer, LifecycleOptions, MotionOptions, Overlay, Removal, ShowOptions,
  components::{CaptureFrame, ClickTarget},
  layers::{Cursor, CursorImage, Outline, Status},
  style::{Insets, OutlineStyle},
};
use auv_driver_common::{Rect, ScreenPoint};

#[test]
fn overlay_preserves_layer_order_and_runtime_svg_without_public_ids() {
  let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><path d="M0 0L8 4L4 8Z"/></svg>"#;
  let overlay = Overlay::new()
    .with_layer(Outline::new(Rect::new(10.0, 20.0, 300.0, 200.0)).with_label("Selected window"))
    .with_layer(Cursor::new(ScreenPoint::new(42.0, 64.0)).with_label("AUV").with_image(CursorImage::svg(svg)));

  assert!(matches!(overlay.layers()[0], Layer::Outline(_)));
  let Layer::Cursor(cursor) = &overlay.layers()[1] else {
    panic!("expected cursor layer");
  };
  assert_eq!(
    cursor.image(),
    &CursorImage::Svg {
      source: svg.to_string()
    }
  );
}

#[test]
fn style_preset_is_refined_before_being_attached_to_a_layer() {
  let style = OutlineStyle::selected().with_padding(Insets::all(4.0));
  let outline = Outline::new(Rect::new(1.0, 2.0, 3.0, 4.0)).with_style(style);

  assert_eq!(outline.style().padding, Insets::all(4.0));
  assert_eq!(outline.style().stroke.color, super::style::Color::AUV_LIME);
}

#[test]
fn with_layer_accepts_primitives_and_composites() {
  let capture = CaptureFrame::new(Rect::new(0.0, 0.0, 100.0, 80.0)).with_label("display");
  let click = ClickTarget::new(ScreenPoint::new(50.0, 40.0))
    .with_outline(Rect::new(10.0, 20.0, 80.0, 30.0))
    .with_outline_label("Quest Start")
    .with_cursor_label("auv · click")
    .with_status("text click delivered");
  let overlay = Overlay::new().with_layer(capture).with_layer(click).with_layer(Status::new(ScreenPoint::new(4.0, 8.0), "done"));

  assert!(matches!(
    overlay.layers(),
    [
      Layer::Outline(_),
      Layer::Outline(_),
      Layer::Cursor(_),
      Layer::Status(_),
      Layer::Status(_)
    ]
  ));
}

#[test]
fn outline_and_cursor_labels_are_hidden_until_explicitly_made_visible() {
  let hidden_outline = Outline::new(Rect::new(1.0, 2.0, 3.0, 4.0)).with_label("target");
  let visible_outline = hidden_outline.clone().with_label_visible();
  assert!(!hidden_outline.label_visible());
  assert!(visible_outline.label_visible());

  let hidden_cursor = Cursor::new(ScreenPoint::new(5.0, 6.0)).with_label("auv · click");
  let visible_cursor = hidden_cursor.clone().with_label_visible();
  assert!(!hidden_cursor.label_visible());
  assert!(visible_cursor.label_visible());
}

#[test]
fn click_target_keeps_outline_and_cursor_labels_independent() {
  let layers = ClickTarget::new(ScreenPoint::new(50.0, 40.0))
    .with_outline(Rect::new(10.0, 20.0, 80.0, 30.0))
    .with_outline_label("Quest Start")
    .with_outline_label_visible()
    .with_cursor_label("auv · click")
    .into_overlay_layers();

  let [Layer::Outline(outline), Layer::Cursor(cursor)] = layers.as_slice() else {
    panic!("expected outline and cursor");
  };
  assert_eq!(outline.label(), Some("Quest Start"));
  assert!(outline.label_visible());
  assert_eq!(cursor.label(), Some("auv · click"));
  assert!(!cursor.label_visible());
}

#[test]
fn click_target_style_configuration_does_not_depend_on_call_order() {
  let style = OutlineStyle::selected().with_padding(Insets::all(7.0));
  let layers = ClickTarget::new(ScreenPoint::new(50.0, 40.0))
    .with_outline_style(style)
    .with_outline(Rect::new(10.0, 20.0, 80.0, 30.0))
    .into_overlay_layers();
  let Layer::Outline(outline) = &layers[0] else {
    panic!("expected outline");
  };
  assert_eq!(outline.style(), style);
}

#[test]
fn show_options_shortcuts_build_the_nested_policies() {
  let options = ShowOptions::new()
    .with_motion_ease(Duration::from_millis(480), Easing::EaseInOutExpo)
    .with_auto_removal_after(Duration::from_millis(240));

  assert_eq!(options.motion().duration(), Duration::from_millis(480));
  assert_eq!(options.motion().easing(), Easing::EaseInOutExpo);
  assert_eq!(options.lifecycle().removal(), Removal::AutoAfter(Duration::from_millis(240)));
}

#[test]
fn show_options_accept_complete_motion_and_lifecycle_policies() {
  let motion = MotionOptions::new().with_duration(Duration::from_millis(80));
  let lifecycle = LifecycleOptions::manual();
  let options = ShowOptions::new().with_motion_options(motion).with_lifecycle_options(lifecycle);

  assert_eq!(options.motion(), motion);
  assert_eq!(options.lifecycle(), lifecycle);
}
