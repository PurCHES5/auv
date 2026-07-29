#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgSpec {
  pub flag: &'static str,
  pub value_name: &'static str,
  pub required: bool,
  pub help: &'static str,
}

pub const TARGET: ArgSpec = ArgSpec {
  flag: "--target",
  value_name: "APP",
  required: false,
  help: "Target application identifier or name.",
};
pub const TITLE: ArgSpec = ArgSpec {
  flag: "--title",
  value_name: "TEXT",
  required: false,
  help: "Window title text used to select the capture target.",
};
pub const LABEL: ArgSpec = ArgSpec {
  flag: "--label",
  value_name: "LABEL",
  required: false,
  help: "Human-readable label for the operation.",
};
pub const QUERY: ArgSpec = ArgSpec {
  flag: "--query",
  value_name: "TEXT",
  required: true,
  help: "Text query used by the invoke command.",
};
pub const OPTIONAL_QUERY: ArgSpec = ArgSpec {
  flag: "--query",
  value_name: "TEXT",
  required: false,
  help: "Text query used by the invoke command; required unless --candidate is supplied.",
};
pub const CANDIDATE: ArgSpec = ArgSpec {
  flag: "--candidate",
  value_name: "JSON",
  required: false,
  help: "Promoted typed candidate JSON consumed instead of --query where supported.",
};
pub const TEXT: ArgSpec = ArgSpec {
  flag: "--text",
  value_name: "TEXT",
  required: true,
  help: "Text content used by the invoke command.",
};
pub const CONTENT: ArgSpec = ArgSpec {
  flag: "--content",
  value_name: "TEXT",
  required: true,
  help: "Document body text to write.",
};
pub const REPLACE: ArgSpec = ArgSpec {
  flag: "--replace",
  value_name: "BOOL",
  required: false,
  help: "Replace existing document body before paste (default true).",
};
pub const VERIFY: ArgSpec = ArgSpec {
  flag: "--verify",
  value_name: "BOOL",
  required: false,
  help: "Run AX text verification after paste (default true).",
};
pub const TARGET_TEXT: ArgSpec = ArgSpec {
  flag: "--target_text",
  value_name: "TEXT",
  required: true,
  help: "Expected AX text used by the invoke command.",
};
pub const IMAGE_PATH: ArgSpec = ArgSpec {
  flag: "--image_path",
  value_name: "PATH",
  required: true,
  help: "Existing image artifact or PNG path to inspect.",
};
pub const KEY: ArgSpec = ArgSpec {
  flag: "--key",
  value_name: "KEY",
  required: true,
  help: "Keyboard key or shortcut to press.",
};
pub const OVERLAY: ArgSpec = ArgSpec {
  flag: "--overlay",
  value_name: "BOOL",
  required: false,
  help: "Draw visual-only overlay cursor evidence where supported.",
};
pub const REGION_X: ArgSpec = ArgSpec {
  flag: "--x",
  value_name: "NUMBER",
  required: true,
  help: "Region left coordinate in the selected coordinate space.",
};
pub const REGION_Y: ArgSpec = ArgSpec {
  flag: "--y",
  value_name: "NUMBER",
  required: true,
  help: "Region top coordinate in the selected coordinate space.",
};
pub const REGION_WIDTH: ArgSpec = ArgSpec {
  flag: "--width",
  value_name: "NUMBER",
  required: true,
  help: "Region width in the selected coordinate space.",
};
pub const REGION_HEIGHT: ArgSpec = ArgSpec {
  flag: "--height",
  value_name: "NUMBER",
  required: true,
  help: "Region height in the selected coordinate space.",
};
pub const OFFSET_X: ArgSpec = ArgSpec {
  flag: "--offset_x",
  value_name: "NUMBER",
  required: false,
  help: "Absolute window-pixel X coordinate for the click target.",
};
pub const OFFSET_Y: ArgSpec = ArgSpec {
  flag: "--offset_y",
  value_name: "NUMBER",
  required: false,
  help: "Absolute window-pixel Y coordinate for the click target.",
};
pub const RELATIVE_X: ArgSpec = ArgSpec {
  flag: "--relative_x",
  value_name: "NUMBER",
  required: false,
  help: "Relative window X coordinate in 0..1 against the resolved window width.",
};
pub const RELATIVE_Y: ArgSpec = ArgSpec {
  flag: "--relative_y",
  value_name: "NUMBER",
  required: false,
  help: "Relative window Y coordinate in 0..1 against the resolved window height.",
};
pub const INPUT_POLICY: ArgSpec = ArgSpec {
  flag: "--input-policy",
  value_name: "POLICY",
  required: false,
  help: "Window input policy: background-only, background-preferred, or foreground-preferred (default background-preferred).",
};
pub const CLICK_COUNT: ArgSpec = ArgSpec {
  flag: "--click-count",
  value_name: "COUNT",
  required: false,
  help: "Number of consecutive clicks from 1 through 255 (default 1).",
};
pub const CLICK_INTERVAL_MS: ArgSpec = ArgSpec {
  flag: "--click-interval-ms",
  value_name: "MILLIS",
  required: false,
  help: "Delay between consecutive clicks in milliseconds (default 75).",
};
pub const PADDING: ArgSpec = ArgSpec {
  flag: "--padding",
  value_name: "POINTS",
  required: false,
  help: "Uniform visual padding in logical points.",
};
pub const BORDER_COLOR: ArgSpec = ArgSpec {
  flag: "--border-color",
  value_name: "HEX",
  required: false,
  help: "Outline color as #RRGGBB or #RRGGBBAA.",
};
pub const BORDER_WIDTH: ArgSpec = ArgSpec {
  flag: "--border-width",
  value_name: "POINTS",
  required: false,
  help: "Outline stroke width in logical points.",
};
pub const FOREGROUND_COLOR: ArgSpec = ArgSpec {
  flag: "--foreground-color",
  value_name: "HEX",
  required: false,
  help: "Text color as #RRGGBB or #RRGGBBAA.",
};
pub const BACKGROUND_COLOR: ArgSpec = ArgSpec {
  flag: "--background-color",
  value_name: "HEX",
  required: false,
  help: "Background color as #RRGGBB or #RRGGBBAA.",
};
pub const CORNER_RADIUS: ArgSpec = ArgSpec {
  flag: "--corner-radius",
  value_name: "POINTS",
  required: false,
  help: "Visual corner radius in logical points.",
};
pub const SPRITE_SIZE: ArgSpec = ArgSpec {
  flag: "--sprite-size",
  value_name: "POINTS",
  required: false,
  help: "Cursor image size in logical points.",
};
pub const SVG: ArgSpec = ArgSpec {
  flag: "--svg",
  value_name: "SOURCE",
  required: false,
  help: "Runtime SVG source used instead of the built-in AUV cursor.",
};
pub const STATUS: ArgSpec = ArgSpec {
  flag: "--status",
  value_name: "TEXT",
  required: false,
  help: "Status text shown beside the target.",
};
pub const MOTION_DURATION_MS: ArgSpec = ArgSpec {
  flag: "--motion-duration-ms",
  value_name: "MILLIS",
  required: false,
  help: "Native appearance and cursor motion duration (default 320).",
};
pub const HOLD_DURATION_MS: ArgSpec = ArgSpec {
  flag: "--hold-duration-ms",
  value_name: "MILLIS",
  required: false,
  help: "Time to keep the overlay visible before hiding (default 2000).",
};
pub const LABEL_VISIBLE: ArgSpec = ArgSpec {
  flag: "--label-visible",
  value_name: "BOOL",
  required: false,
  help: "Render the configured layer label (default false).",
};
pub const OUTLINE_LABEL: ArgSpec = ArgSpec {
  flag: "--outline-label",
  value_name: "TEXT",
  required: false,
  help: "Text stored for the click-target outline label.",
};
pub const OUTLINE_LABEL_VISIBLE: ArgSpec = ArgSpec {
  flag: "--outline-label-visible",
  value_name: "BOOL",
  required: false,
  help: "Render the click-target outline label (default false).",
};
pub const CURSOR_LABEL: ArgSpec = ArgSpec {
  flag: "--cursor-label",
  value_name: "TEXT",
  required: false,
  help: "Text stored for the click-target cursor label.",
};
pub const CURSOR_LABEL_VISIBLE: ArgSpec = ArgSpec {
  flag: "--cursor-label-visible",
  value_name: "BOOL",
  required: false,
  help: "Render the click-target cursor label (default false).",
};
pub const OUTLINE_PADDING: ArgSpec = ArgSpec {
  flag: "--outline-padding",
  value_name: "POINTS",
  required: false,
  help: "Click-target outline padding in logical points.",
};
pub const OUTLINE_CORNER_RADIUS: ArgSpec = ArgSpec {
  flag: "--outline-corner-radius",
  value_name: "POINTS",
  required: false,
  help: "Click-target outline corner radius in logical points.",
};
pub const STATUS_PADDING: ArgSpec = ArgSpec {
  flag: "--status-padding",
  value_name: "POINTS",
  required: false,
  help: "Click-target status padding in logical points.",
};
pub const STATUS_CORNER_RADIUS: ArgSpec = ArgSpec {
  flag: "--status-corner-radius",
  value_name: "POINTS",
  required: false,
  help: "Click-target status corner radius in logical points.",
};
pub const STATUS_FOREGROUND_COLOR: ArgSpec = ArgSpec {
  flag: "--status-foreground-color",
  value_name: "HEX",
  required: false,
  help: "Click-target status text color as #RRGGBB or #RRGGBBAA.",
};
pub const STATUS_BACKGROUND_COLOR: ArgSpec = ArgSpec {
  flag: "--status-background-color",
  value_name: "HEX",
  required: false,
  help: "Click-target status background as #RRGGBB or #RRGGBBAA.",
};

pub const TARGET_ARGS: &[ArgSpec] = &[TARGET];
pub const WINDOW_ARGS: &[ArgSpec] = &[TARGET, TITLE];
pub const WINDOW_CLICK_POINT_ARGS: &[ArgSpec] = &[
  TARGET,
  TITLE,
  OFFSET_X,
  OFFSET_Y,
  RELATIVE_X,
  RELATIVE_Y,
  INPUT_POLICY,
  CLICK_COUNT,
  CLICK_INTERVAL_MS,
];
pub const LABEL_ARGS: &[ArgSpec] = &[TARGET, LABEL];
pub const SCREEN_TEXT_ARGS: &[ArgSpec] = &[TARGET, QUERY];
pub const IMAGE_TEXT_ARGS: &[ArgSpec] = &[QUERY, IMAGE_PATH];
pub const TEXT_ARGS: &[ArgSpec] = &[TARGET, TEXT];
pub const KEY_ARGS: &[ArgSpec] = &[TARGET, KEY];
pub const QUERY_ARGS: &[ArgSpec] = &[TARGET, QUERY];
pub const WINDOW_TEXT_ARGS: &[ArgSpec] = &[TARGET, TITLE, QUERY];
pub const WINDOW_CLICK_TEXT_ARGS: &[ArgSpec] = &[
  TARGET,
  TITLE,
  QUERY,
  INPUT_POLICY,
  CLICK_COUNT,
  CLICK_INTERVAL_MS,
];
pub const WINDOW_VERIFY_TEXT_ARGS: &[ArgSpec] = &[TARGET, TARGET_TEXT];
pub const QUERY_OR_CANDIDATE_ARGS: &[ArgSpec] = &[TARGET, OPTIONAL_QUERY, CANDIDATE];
pub const QUERY_OVERLAY_ARGS: &[ArgSpec] = &[TARGET, QUERY, OVERLAY];
pub const QUERY_OR_CANDIDATE_OVERLAY_ARGS: &[ArgSpec] = &[TARGET, OPTIONAL_QUERY, CANDIDATE, OVERLAY];
pub const WINDOW_QUERY_OVERLAY_ARGS: &[ArgSpec] = &[TARGET, TITLE, QUERY, OVERLAY];
pub const REGION_ARGS: &[ArgSpec] = &[
  TARGET,
  REGION_X,
  REGION_Y,
  REGION_WIDTH,
  REGION_HEIGHT,
  LABEL,
];
pub const OVERLAY_OUTLINE_ARGS: &[ArgSpec] = &[
  REGION_X,
  REGION_Y,
  REGION_WIDTH,
  REGION_HEIGHT,
  LABEL,
  LABEL_VISIBLE,
  PADDING,
  BORDER_COLOR,
  BORDER_WIDTH,
  CORNER_RADIUS,
  MOTION_DURATION_MS,
  HOLD_DURATION_MS,
];
pub const OVERLAY_CURSOR_ARGS: &[ArgSpec] = &[
  REGION_X,
  REGION_Y,
  LABEL,
  LABEL_VISIBLE,
  SVG,
  PADDING,
  FOREGROUND_COLOR,
  BACKGROUND_COLOR,
  CORNER_RADIUS,
  SPRITE_SIZE,
  MOTION_DURATION_MS,
  HOLD_DURATION_MS,
];
pub const OVERLAY_STATUS_ARGS: &[ArgSpec] = &[
  REGION_X,
  REGION_Y,
  TEXT,
  PADDING,
  FOREGROUND_COLOR,
  BACKGROUND_COLOR,
  CORNER_RADIUS,
  MOTION_DURATION_MS,
  HOLD_DURATION_MS,
];
pub const OVERLAY_CLICK_TARGET_ARGS: &[ArgSpec] = &[
  REGION_X,
  REGION_Y,
  REGION_WIDTH,
  REGION_HEIGHT,
  OUTLINE_LABEL,
  OUTLINE_LABEL_VISIBLE,
  CURSOR_LABEL,
  CURSOR_LABEL_VISIBLE,
  STATUS,
  OUTLINE_PADDING,
  BORDER_COLOR,
  BORDER_WIDTH,
  OUTLINE_CORNER_RADIUS,
  STATUS_PADDING,
  STATUS_FOREGROUND_COLOR,
  STATUS_BACKGROUND_COLOR,
  STATUS_CORNER_RADIUS,
  MOTION_DURATION_MS,
  HOLD_DURATION_MS,
];
pub const FIXTURE_DIR: ArgSpec = ArgSpec {
  flag: "--fixture-dir",
  value_name: "PATH",
  required: true,
  help: "Directory containing a single-frame scan fixture (manifest.json + PNG).",
};
pub const SCAN_FRAME_ARGS: &[ArgSpec] = &[FIXTURE_DIR];
pub const COVERAGE_FIXTURE_DIR: ArgSpec = ArgSpec {
  flag: "--fixture-dir",
  value_name: "PATH",
  required: true,
  help: "Directory containing a coverage scenario manifest (manifest.json); frame PNGs are resolved via frame_fixture cross-reference, not stored in this directory.",
};
pub const SCAN_COVERAGE_ARGS: &[ArgSpec] = &[COVERAGE_FIXTURE_DIR];
pub const TEXTEDIT_DOCUMENT_WRITE_ARGS: &[ArgSpec] = &[CONTENT, REPLACE, VERIFY];
pub const NO_ARGS: &[ArgSpec] = &[];
