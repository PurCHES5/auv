use std::time::Duration;

use crate::{
  CommandGroup, InvokeCommandInput, InvokeCommandOutput, InvokeCommandResult, InvokeReport, InvokeReportField,
  arg::{OVERLAY_CLICK_TARGET_ARGS, OVERLAY_CURSOR_ARGS, OVERLAY_OUTLINE_ARGS, OVERLAY_STATUS_ARGS},
  invoke_command,
};
use auv_driver::overlay::{
  Easing, Overlay, ShowOptions,
  components::{CaptureFrame, ClickTarget},
  layers::{Cursor, CursorImage, Outline, Status},
  style::{Color, CursorStyle, Insets, OutlineStyle, StatusStyle, Stroke},
};
use auv_driver::{Rect, ScreenPoint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OverlayStatus {
  Disabled,
  #[cfg(all(target_os = "macos", feature = "overlay"))]
  Shown {
    layers: usize,
  },
  Unavailable {
    reason: String,
  },
}

impl OverlayStatus {
  pub(crate) fn report_field(&self) -> InvokeReportField {
    match self {
      Self::Disabled => InvokeReportField::new("Overlay", "disabled"),
      #[cfg(all(target_os = "macos", feature = "overlay"))]
      Self::Shown { layers } => InvokeReportField::new("Overlay", format!("shown ({layers} layers)")),
      Self::Unavailable { reason } => InvokeReportField::new("Overlay", format!("unavailable: {reason}")),
    }
  }
}

pub(crate) fn show_overlay(
  input: &InvokeCommandInput,
  session: &auv_driver::LocalDriverSession,
  overlay: Overlay,
  options: ShowOptions,
) -> Result<OverlayStatus, String> {
  if input.dry_run || !input.overlay_enabled()? {
    return Ok(OverlayStatus::Disabled);
  }

  #[cfg(all(target_os = "macos", feature = "overlay"))]
  {
    let layers = overlay.layers().len();
    return Ok(match session.overlay().show(&overlay, options) {
      Ok(()) => OverlayStatus::Shown { layers },
      Err(error) => OverlayStatus::Unavailable {
        reason: error.to_string(),
      },
    });
  }

  #[cfg(not(all(target_os = "macos", feature = "overlay")))]
  {
    let _ = (session, overlay, options);
    Ok(OverlayStatus::Unavailable {
      reason: "the local driver has no compiled overlay adapter".to_string(),
    })
  }
}

pub fn group() -> CommandGroup {
  CommandGroup::new("overlay", "OVERLAY")
    .command(show_outline_invoke_command())
    .command(show_cursor_invoke_command())
    .command(show_status_invoke_command())
    .command(show_capture_frame_invoke_command())
    .command(show_click_target_invoke_command())
}

#[invoke_command(
  id = "overlay.outline",
  group = "overlay",
  description = "Present one configurable outline layer for visual style inspection.",
  args = OVERLAY_OUTLINE_ARGS,
)]
async fn show_outline(input: InvokeCommandInput) -> InvokeCommandResult {
  let rect = rect_input(&input)?;
  let mut outline = Outline::new(rect).with_style(outline_style(&input, OutlineStyle::new(), "padding", "corner-radius")?);
  if let Some(label) = input.inputs.get("label") {
    outline = outline.with_label(label.clone());
  }
  if optional_bool(&input, "label-visible", false)? {
    outline = outline.with_label_visible();
  }
  debug_output(&input, "Outline", Overlay::new().with_layer(outline))
}

#[invoke_command(
  id = "overlay.cursor",
  group = "overlay",
  description = "Present one configurable cursor layer, optionally using runtime SVG source.",
  args = OVERLAY_CURSOR_ARGS,
)]
async fn show_cursor(input: InvokeCommandInput) -> InvokeCommandResult {
  let mut cursor = Cursor::new(point_input(&input)?).with_style(cursor_style(&input)?);
  if let Some(label) = input.inputs.get("label") {
    cursor = cursor.with_label(label.clone());
  }
  if optional_bool(&input, "label-visible", false)? {
    cursor = cursor.with_label_visible();
  }
  if let Some(svg) = input.inputs.get("svg") {
    cursor = cursor.with_image(CursorImage::svg(svg.clone()));
  }
  debug_output(&input, "Cursor", Overlay::new().with_layer(cursor))
}

#[invoke_command(
  id = "overlay.status",
  group = "overlay",
  description = "Present one configurable status layer for visual style inspection.",
  args = OVERLAY_STATUS_ARGS,
)]
async fn show_status(input: InvokeCommandInput) -> InvokeCommandResult {
  let status = Status::new(point_input(&input)?, input.required_input("text")?).with_style(status_style(
    &input,
    "padding",
    "foreground-color",
    "background-color",
    "corner-radius",
  )?);
  debug_output(&input, "Status", Overlay::new().with_layer(status))
}

#[invoke_command(
  id = "overlay.captureFrame",
  group = "overlay",
  description = "Present the reusable capture-frame component around a screen rectangle.",
  args = OVERLAY_OUTLINE_ARGS,
)]
async fn show_capture_frame(input: InvokeCommandInput) -> InvokeCommandResult {
  let mut frame =
    CaptureFrame::new(rect_input(&input)?).with_style(outline_style(&input, OutlineStyle::capture(), "padding", "corner-radius")?);
  if let Some(label) = input.inputs.get("label") {
    frame = frame.with_label(label.clone());
  }
  if optional_bool(&input, "label-visible", false)? {
    frame = frame.with_label_visible();
  }
  debug_output(&input, "CaptureFrame", Overlay::new().with_layer(frame))
}

#[invoke_command(
  id = "overlay.clickTarget",
  group = "overlay",
  description = "Present the reusable click-target component with outline, cursor, and status layers.",
  args = OVERLAY_CLICK_TARGET_ARGS,
)]
async fn show_click_target(input: InvokeCommandInput) -> InvokeCommandResult {
  let rect = rect_input(&input)?;
  let point = ScreenPoint::new(rect.origin.x + rect.size.width / 2.0, rect.origin.y + rect.size.height / 2.0);
  let mut target = ClickTarget::new(point)
    .with_outline(rect)
    .with_outline_style(outline_style(&input, OutlineStyle::selected(), "outline-padding", "outline-corner-radius")?)
    .with_status(input.inputs.get("status").map(String::as_str).unwrap_or("click target"));
  if let Some(label) = input.inputs.get("outline-label") {
    target = target.with_outline_label(label.clone());
  }
  if optional_bool(&input, "outline-label-visible", false)? {
    target = target.with_outline_label_visible();
  }
  if let Some(label) = input.inputs.get("cursor-label") {
    target = target.with_cursor_label(label.clone());
  }
  if optional_bool(&input, "cursor-label-visible", false)? {
    target = target.with_cursor_label_visible();
  }
  target = target.with_status_style(status_style(
    &input,
    "status-padding",
    "status-foreground-color",
    "status-background-color",
    "status-corner-radius",
  )?);
  debug_output(&input, "ClickTarget", Overlay::new().with_layer(target))
}

fn debug_output(input: &InvokeCommandInput, component: &str, overlay: Overlay) -> InvokeCommandResult {
  let layers = overlay.layers().len();
  let options = show_options(input)?;
  let status = if input.dry_run || !input.overlay_enabled()? {
    OverlayStatus::Disabled
  } else {
    #[cfg(all(target_os = "macos", feature = "overlay"))]
    {
      match auv_driver::open_local() {
        Ok(session) => show_overlay(input, &session, overlay, options)?,
        Err(error) => OverlayStatus::Unavailable {
          reason: error.to_string(),
        },
      }
    }
    #[cfg(not(all(target_os = "macos", feature = "overlay")))]
    {
      let _ = (overlay, options);
      OverlayStatus::Unavailable {
        reason: "the local driver has no compiled overlay adapter".to_string(),
      }
    }
  };
  if let OverlayStatus::Unavailable { reason } = &status {
    return Err(format!("overlay.{component} could not be shown: {reason}"));
  }
  Ok(InvokeCommandOutput::completed().with_report(InvokeReport::new(
    vec![
      InvokeReportField::new("Component", component),
      InvokeReportField::new("Layers", layers.to_string()),
      InvokeReportField::new("Motion", format!("{} ms", options.motion().duration().as_millis())),
      InvokeReportField::new("Hold", format!("{} ms", auto_removal_delay(options).as_millis())),
      status.report_field(),
    ],
    Vec::new(),
  )))
}

fn show_options(input: &InvokeCommandInput) -> Result<ShowOptions, String> {
  Ok(
    ShowOptions::new()
      .with_motion_ease(Duration::from_millis(optional_u64(input, "motion-duration-ms", 320)?), Easing::EaseInOutExpo)
      .with_auto_removal_after(Duration::from_millis(optional_u64(input, "hold-duration-ms", 2_000)?)),
  )
}

fn auto_removal_delay(options: ShowOptions) -> Duration {
  match options.lifecycle().removal() {
    auv_driver::overlay::Removal::AutoAfter(duration) => duration,
    auv_driver::overlay::Removal::Manual => Duration::ZERO,
  }
}

fn rect_input(input: &InvokeCommandInput) -> Result<Rect, String> {
  let x = required_f64(input, "x")?;
  let y = required_f64(input, "y")?;
  let width = required_non_negative_f64(input, "width")?;
  let height = required_non_negative_f64(input, "height")?;
  if width == 0.0 || height == 0.0 {
    return Err(format!("{} requires positive --width and --height", input.command_id));
  }
  Ok(Rect::new(x, y, width, height))
}

fn point_input(input: &InvokeCommandInput) -> Result<ScreenPoint, String> {
  Ok(ScreenPoint::new(required_f64(input, "x")?, required_f64(input, "y")?))
}

fn outline_style(
  input: &InvokeCommandInput,
  mut style: OutlineStyle,
  padding_name: &str,
  corner_radius_name: &str,
) -> Result<OutlineStyle, String> {
  if let Some(value) = optional_non_negative_f64(input, padding_name)? {
    style = style.with_padding(Insets::all(value));
  }
  if input.inputs.contains_key("border-color") || input.inputs.contains_key("border-width") {
    let color = optional_color(input, "border-color")?.unwrap_or(style.stroke.color);
    let width = optional_non_negative_f64(input, "border-width")?.unwrap_or(style.stroke.width);
    style = style.with_stroke(Stroke::new(color, width));
  }
  if let Some(value) = optional_non_negative_f64(input, corner_radius_name)? {
    style = style.with_corner_radius(value);
  }
  Ok(style)
}

fn cursor_style(input: &InvokeCommandInput) -> Result<CursorStyle, String> {
  let mut style = CursorStyle::auv();
  if let Some(value) = optional_non_negative_f64(input, "padding")? {
    style = style.with_label_padding(Insets::all(value));
  }
  if let Some(color) = optional_color(input, "foreground-color")? {
    style = style.with_label_foreground(color);
  }
  if let Some(color) = optional_color(input, "background-color")? {
    style = style.with_label_background(color);
  }
  if let Some(value) = optional_non_negative_f64(input, "corner-radius")? {
    style = style.with_label_corner_radius(value);
  }
  if let Some(value) = optional_non_negative_f64(input, "sprite-size")? {
    if value == 0.0 {
      return Err(format!("{} requires --sprite-size greater than zero", input.command_id));
    }
    style = style.with_sprite_size(value);
  }
  Ok(style)
}

fn status_style(
  input: &InvokeCommandInput,
  padding_name: &str,
  foreground_name: &str,
  background_name: &str,
  corner_radius_name: &str,
) -> Result<StatusStyle, String> {
  let mut style = StatusStyle::action();
  if let Some(value) = optional_non_negative_f64(input, padding_name)? {
    style = style.with_padding(Insets::all(value));
  }
  if let Some(color) = optional_color(input, foreground_name)? {
    style = style.with_foreground(color);
  }
  if let Some(color) = optional_color(input, background_name)? {
    style = style.with_background(color);
  }
  if let Some(value) = optional_non_negative_f64(input, corner_radius_name)? {
    style = style.with_corner_radius(value);
  }
  Ok(style)
}

fn required_f64(input: &InvokeCommandInput, name: &str) -> Result<f64, String> {
  parse_f64(input, name, input.required_input(name)?)
}

fn required_non_negative_f64(input: &InvokeCommandInput, name: &str) -> Result<f64, String> {
  let value = required_f64(input, name)?;
  ensure_non_negative(input, name, value)
}

fn optional_non_negative_f64(input: &InvokeCommandInput, name: &str) -> Result<Option<f64>, String> {
  input.inputs.get(name).map(|raw| parse_f64(input, name, raw).and_then(|value| ensure_non_negative(input, name, value))).transpose()
}

fn parse_f64(input: &InvokeCommandInput, name: &str, raw: &str) -> Result<f64, String> {
  let value = raw.parse::<f64>().map_err(|error| format!("{} received invalid --{name} value {raw:?}: {error}", input.command_id))?;
  if !value.is_finite() {
    return Err(format!("{} requires finite --{name}", input.command_id));
  }
  Ok(value)
}

fn ensure_non_negative(input: &InvokeCommandInput, name: &str, value: f64) -> Result<f64, String> {
  if value < 0.0 {
    Err(format!("{} requires non-negative --{name}", input.command_id))
  } else {
    Ok(value)
  }
}

fn optional_u64(input: &InvokeCommandInput, name: &str, default: u64) -> Result<u64, String> {
  input.inputs.get(name).map_or(Ok(default), |raw| {
    raw.parse::<u64>().map_err(|error| format!("{} received invalid --{name} value {raw:?}: {error}", input.command_id))
  })
}

fn optional_bool(input: &InvokeCommandInput, name: &str, default: bool) -> Result<bool, String> {
  input.inputs.get(name).map_or(Ok(default), |raw| {
    raw.parse::<bool>().map_err(|error| format!("{} received invalid --{name} value {raw:?}: {error}", input.command_id))
  })
}

fn optional_color(input: &InvokeCommandInput, name: &str) -> Result<Option<Color>, String> {
  input.inputs.get(name).map(|raw| parse_color(&input.command_id, name, raw)).transpose()
}

fn parse_color(command_id: &str, name: &str, raw: &str) -> Result<Color, String> {
  let hex = raw.strip_prefix('#').unwrap_or(raw);
  if hex.len() != 6 && hex.len() != 8 {
    return Err(format!("{command_id} requires --{name} as #RRGGBB or #RRGGBBAA"));
  }
  let byte = |range: std::ops::Range<usize>| {
    u8::from_str_radix(&hex[range], 16).map_err(|_| format!("{command_id} received invalid --{name} color {raw:?}"))
  };
  let red = byte(0..2)?;
  let green = byte(2..4)?;
  let blue = byte(4..6)?;
  let alpha = if hex.len() == 8 { byte(6..8)? } else { 255 };
  Ok(Color::rgba(f64::from(red) / 255.0, f64::from(green) / 255.0, f64::from(blue) / 255.0, f64::from(alpha) / 255.0))
}

#[cfg(test)]
#[path = "overlay_test.rs"]
mod tests;
