//! Hierarchical clients for one routed Driver Runner.
//!
//! Each child keeps the route and any resolved resource reference. The
//! public hierarchy is independent of whether the daemon reaches a local child
//! process or a paired remote Device.

use auv_api_proto::auv::api::driver::macos::v1 as macos_proto;
use auv_api_proto::auv::api::driver::v1 as proto;
use auv_api_proto::auv::api::image::v1::NormalizedRect;

use auv_api_client::protocol::grpc::Client as GrpcClient;

// Placement is selected by `Client`/`RunClient` before this route-bound
// hierarchy is constructed. `Client::local()` is the explicit local-only
// constraint; ordinary placement may resolve either a local or paired Device.

#[derive(Clone, Debug)]
pub struct RunnerClient {
  client: GrpcClient,
  route: auv_api_client::RunnerRoute,
}

impl RunnerClient {
  pub(crate) fn new(client: GrpcClient, route: auv_api_client::RunnerRoute) -> Result<Self, tonic::Status> {
    if route.runner_class.trim().is_empty() {
      return Err(tonic::Status::invalid_argument("Runner route must include runner_class"));
    }
    Ok(Self { client, route })
  }

  /// Builds the routed transport for an application-owned generated protobuf
  /// client while keeping daemon lifecycle resources out of that application's
  /// messages and metadata.
  pub fn transport(&self) -> Result<auv_api_client::RoutedTransport, tonic::Status> {
    self.client.routed_transport(self.route.clone())
  }

  pub fn displays(&self) -> DisplaysClient {
    DisplaysClient {
      runner: self.clone(),
    }
  }

  pub fn windows(&self) -> WindowsClient {
    WindowsClient {
      runner: self.clone(),
    }
  }

  pub fn input(&self) -> InputClient {
    InputClient {
      runner: self.clone(),
    }
  }

  pub fn overlay(&self) -> OverlayClient {
    OverlayClient {
      runner: self.clone(),
    }
  }

  pub fn macos(&self) -> MacosClient {
    MacosClient {
      runner: self.clone(),
    }
  }

  /// Runs OCR against a capture already obtained from this Runner.
  pub async fn recognize_text(
    &self,
    capture: proto::CapturedFrame,
    region: Option<NormalizedRect>,
    custom_words: Vec<String>,
    recognition_languages: Vec<String>,
  ) -> Result<proto::RecognizeTextResponse, tonic::Status> {
    Ok(
      proto::text_recognition_service_client::TextRecognitionServiceClient::new(self.transport()?)
        .recognize_text(proto::RecognizeTextRequest {
          capture: Some(capture),
          region,
          custom_words,
          recognition_languages,
        })
        .await?
        .into_inner(),
    )
  }
}

#[derive(Clone, Debug)]
pub struct OverlayClient {
  runner: RunnerClient,
}

impl OverlayClient {
  pub async fn show(
    &self,
    overlay: &auv_driver_overlay_common::Overlay,
    options: auv_driver_overlay_common::ShowOptions,
  ) -> Result<(), tonic::Status> {
    proto::overlay_service_client::OverlayServiceClient::new(self.runner.transport()?)
      .show_overlay(proto::ShowOverlayRequest {
        overlay: Some(overlay_to_proto(overlay)?),
        options: Some(overlay_options_to_proto(options)?),
      })
      .await?;
    Ok(())
  }

  pub async fn remove(&self) -> Result<(), tonic::Status> {
    proto::overlay_service_client::OverlayServiceClient::new(self.runner.transport()?)
      .remove_overlay(proto::RemoveOverlayRequest {})
      .await?;
    Ok(())
  }
}

fn overlay_to_proto(value: &auv_driver_overlay_common::Overlay) -> Result<proto::Overlay, tonic::Status> {
  use auv_driver_overlay_common::Layer;
  let layers = value
    .layers()
    .iter()
    .map(|layer| {
      let layer = match layer {
        Layer::Cursor(value) => proto::overlay_layer::Layer::Cursor(proto::Cursor {
          point: Some(proto::ScreenPoint {
            x: value.point().point().x,
            y: value.point().point().y,
          }),
          label: value.label().map(ToOwned::to_owned),
          label_visible: value.label_visible(),
          image: Some(cursor_image_to_proto(value.image())?),
          style: Some(cursor_style_to_proto(value.style())),
        }),
        Layer::Outline(value) => proto::overlay_layer::Layer::Outline(proto::Outline {
          rect: Some(proto::ScreenRect {
            x: value.rect().origin.x,
            y: value.rect().origin.y,
            width: value.rect().size.width,
            height: value.rect().size.height,
          }),
          label: value.label().map(ToOwned::to_owned),
          label_visible: value.label_visible(),
          style: Some(outline_style_to_proto(value.style())),
        }),
        Layer::Status(value) => proto::overlay_layer::Layer::Status(proto::Status {
          point: Some(proto::ScreenPoint {
            x: value.point().point().x,
            y: value.point().point().y,
          }),
          text: value.text().to_string(),
          style: Some(status_style_to_proto(value.style())),
        }),
      };
      Ok(proto::OverlayLayer { layer: Some(layer) })
    })
    .collect::<Result<Vec<_>, tonic::Status>>()?;
  Ok(proto::Overlay { layers })
}

fn cursor_image_to_proto(value: &auv_driver_overlay_common::layers::CursorImage) -> Result<proto::CursorImage, tonic::Status> {
  use auv_driver_overlay_common::layers::{BuiltInCursor, CursorImage};
  let image = match value {
    CursorImage::BuiltIn { variant } => proto::cursor_image::Image::BuiltIn(match variant {
      BuiltInCursor::Auv => proto::BuiltInCursor::Auv as i32,
      BuiltInCursor::AuvClick => proto::BuiltInCursor::AuvClick as i32,
      BuiltInCursor::You => proto::BuiltInCursor::You as i32,
    }),
    CursorImage::Svg { source } if source.len() <= 256 * 1024 => proto::cursor_image::Image::Svg(source.clone()),
    CursorImage::Svg { .. } => return Err(tonic::Status::invalid_argument("cursor SVG exceeds 256 KiB")),
  };
  Ok(proto::CursorImage { image: Some(image) })
}

fn color_to_proto(value: auv_driver_overlay_common::style::Color) -> proto::Color {
  proto::Color {
    red: value.red,
    green: value.green,
    blue: value.blue,
    alpha: value.alpha,
  }
}

fn insets_to_proto(value: auv_driver_overlay_common::style::Insets) -> proto::Insets {
  proto::Insets {
    top: value.top,
    right: value.right,
    bottom: value.bottom,
    left: value.left,
  }
}

fn outline_style_to_proto(value: auv_driver_overlay_common::style::OutlineStyle) -> proto::OutlineStyle {
  proto::OutlineStyle {
    stroke: Some(proto::Stroke {
      color: Some(color_to_proto(value.stroke.color)),
      width: value.stroke.width,
    }),
    padding: Some(insets_to_proto(value.padding)),
    corner_radius: value.corner_radius,
  }
}

fn cursor_style_to_proto(value: auv_driver_overlay_common::style::CursorStyle) -> proto::CursorStyle {
  proto::CursorStyle {
    label_foreground: Some(color_to_proto(value.label_foreground)),
    label_background: Some(color_to_proto(value.label_background)),
    label_padding: Some(insets_to_proto(value.label_padding)),
    label_corner_radius: value.label_corner_radius,
    sprite_size: value.sprite_size,
    label_gap: value.label_gap,
  }
}

fn status_style_to_proto(value: auv_driver_overlay_common::style::StatusStyle) -> proto::StatusStyle {
  proto::StatusStyle {
    foreground: Some(color_to_proto(value.foreground)),
    background: Some(color_to_proto(value.background)),
    padding: Some(insets_to_proto(value.padding)),
    corner_radius: value.corner_radius,
  }
}

fn overlay_options_to_proto(value: auv_driver_overlay_common::ShowOptions) -> Result<proto::ShowOptions, tonic::Status> {
  let duration = |value: std::time::Duration| -> Result<prost_types::Duration, tonic::Status> {
    Ok(prost_types::Duration {
      seconds: i64::try_from(value.as_secs()).map_err(|_| tonic::Status::invalid_argument("overlay duration is too large"))?,
      nanos: value.subsec_nanos() as i32,
    })
  };
  let removal = match value.lifecycle().removal() {
    auv_driver_overlay_common::Removal::Manual => proto::lifecycle_options::Removal::Manual(()),
    auv_driver_overlay_common::Removal::AutoAfter(value) => proto::lifecycle_options::Removal::AutoAfter(duration(value)?),
  };
  Ok(proto::ShowOptions {
    motion: Some(proto::MotionOptions {
      duration: Some(duration(value.motion().duration())?),
      easing: Some(proto::Easing::EaseInOutExpo as i32),
    }),
    lifecycle: Some(proto::LifecycleOptions {
      removal: Some(removal),
    }),
  })
}

#[derive(Clone, Debug)]
pub struct DisplaysClient {
  runner: RunnerClient,
}

impl DisplaysClient {
  pub async fn list(&self) -> Result<Vec<proto::Display>, tonic::Status> {
    Ok(
      proto::display_service_client::DisplayServiceClient::new(self.runner.transport()?)
        .list_displays(proto::ListDisplaysRequest {})
        .await?
        .into_inner()
        .displays,
    )
  }

  pub async fn capture(&self, selector: Option<proto::DisplaySelector>) -> Result<proto::CaptureDisplayResponse, tonic::Status> {
    Ok(
      proto::capture_service_client::CaptureServiceClient::new(self.runner.transport()?)
        .capture_display(proto::CaptureDisplayRequest { selector })
        .await?
        .into_inner(),
    )
  }

  pub async fn capture_region(
    &self,
    region: proto::ScreenRect,
    selector: Option<proto::DisplaySelector>,
  ) -> Result<proto::CaptureRegionResponse, tonic::Status> {
    Ok(
      proto::capture_service_client::CaptureServiceClient::new(self.runner.transport()?)
        .capture_region(proto::CaptureRegionRequest {
          region: Some(region),
          selector,
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn find_text(
    &self,
    selector: Option<proto::DisplaySelector>,
    query: impl Into<String>,
  ) -> Result<proto::FindDisplayTextResponse, tonic::Status> {
    self.find_text_with(selector, query, FindTextOptions::default()).await
  }

  pub async fn find_text_with(
    &self,
    selector: Option<proto::DisplaySelector>,
    query: impl Into<String>,
    options: FindTextOptions,
  ) -> Result<proto::FindDisplayTextResponse, tonic::Status> {
    Ok(
      proto::text_recognition_service_client::TextRecognitionServiceClient::new(self.runner.transport()?)
        .find_display_text(proto::FindDisplayTextRequest {
          selector,
          query: query.into(),
          region: options.region,
          custom_words: options.custom_words,
          recognition_languages: options.recognition_languages,
        })
        .await?
        .into_inner(),
    )
  }
}

#[derive(Clone, Debug)]
pub struct WindowsClient {
  runner: RunnerClient,
}

impl WindowsClient {
  pub async fn list(&self) -> Result<Vec<proto::Window>, tonic::Status> {
    Ok(
      proto::window_service_client::WindowServiceClient::new(self.runner.transport()?)
        .list_windows(proto::ListWindowsRequest {})
        .await?
        .into_inner()
        .windows,
    )
  }

  pub async fn resolve(&self, selector: proto::WindowSelector) -> Result<WindowClient, tonic::Status> {
    let window = proto::window_service_client::WindowServiceClient::new(self.runner.transport()?)
      .resolve_window(proto::ResolveWindowRequest {
        selector: Some(selector),
      })
      .await?
      .into_inner()
      .window
      .ok_or_else(|| tonic::Status::data_loss("ResolveWindow response omitted Window"))?;
    let window_ref = window
      .r#ref
      .clone()
      .filter(|window_ref| !window_ref.window_id.trim().is_empty())
      .ok_or_else(|| tonic::Status::internal("ResolveWindow response omitted WindowRef"))?;
    Ok(WindowClient {
      runner: self.runner.clone(),
      window,
      window_ref,
    })
  }
}

#[derive(Clone, Debug)]
pub struct WindowClient {
  runner: RunnerClient,
  window: proto::Window,
  window_ref: proto::WindowRef,
}

impl WindowClient {
  pub fn resource(&self) -> &proto::Window {
    &self.window
  }

  pub fn reference(&self) -> &proto::WindowRef {
    &self.window_ref
  }

  pub async fn capture(&self) -> Result<proto::CaptureWindowResponse, tonic::Status> {
    Ok(
      proto::capture_service_client::CaptureServiceClient::new(self.runner.transport()?)
        .capture_window(proto::CaptureWindowRequest {
          window: Some(self.window_ref.clone()),
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn find_text(&self, query: impl Into<String>) -> Result<proto::FindWindowTextResponse, tonic::Status> {
    self.find_text_with(query, FindTextOptions::default()).await
  }

  pub async fn find_text_with(
    &self,
    query: impl Into<String>,
    options: FindTextOptions,
  ) -> Result<proto::FindWindowTextResponse, tonic::Status> {
    Ok(
      proto::text_recognition_service_client::TextRecognitionServiceClient::new(self.runner.transport()?)
        .find_window_text(proto::FindWindowTextRequest {
          window: Some(self.window_ref.clone()),
          query: query.into(),
          region: options.region,
          custom_words: options.custom_words,
          recognition_languages: options.recognition_languages,
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn click(
    &self,
    point: proto::WindowPoint,
    options: Option<proto::ClickOptions>,
  ) -> Result<proto::ClickWindowPointResponse, tonic::Status> {
    Ok(
      proto::input_service_client::InputServiceClient::new(self.runner.transport()?)
        .click_window_point(proto::ClickWindowPointRequest {
          window: Some(self.window_ref.clone()),
          point: Some(point),
          options,
        })
        .await?
        .into_inner(),
    )
  }
}

#[derive(Clone, Debug)]
pub struct InputClient {
  runner: RunnerClient,
}

#[derive(Clone, Debug)]
pub struct MacosClient {
  runner: RunnerClient,
}

impl MacosClient {
  pub fn permissions(&self) -> PermissionClient {
    PermissionClient {
      runner: self.runner.clone(),
    }
  }

  pub fn media(&self) -> MediaControlClient {
    MediaControlClient {
      runner: self.runner.clone(),
    }
  }

  pub fn applications(&self) -> ApplicationClient {
    ApplicationClient {
      runner: self.runner.clone(),
    }
  }

  pub fn accessibility(&self) -> AccessibilityClient {
    AccessibilityClient {
      runner: self.runner.clone(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct PermissionClient {
  runner: RunnerClient,
}

#[derive(Clone, Debug)]
pub struct MediaControlClient {
  runner: RunnerClient,
}

#[derive(Clone, Debug)]
pub struct ApplicationClient {
  runner: RunnerClient,
}

#[derive(Clone, Debug)]
pub struct AccessibilityClient {
  runner: RunnerClient,
}

impl AccessibilityClient {
  pub async fn focus_text(&self, options: auv_driver::FocusTextOptions) -> Result<auv_driver::AxFocusResult, tonic::Status> {
    let selector = match options.selector {
      auv_driver::AxTextSelector::Query(query) => macos_proto::focus_text_request::Selector::Query(query),
      auv_driver::AxTextSelector::Path(path) => macos_proto::focus_text_request::Selector::Path(path),
    };
    let response = macos_proto::accessibility_service_client::AccessibilityServiceClient::new(self.runner.transport()?)
      .focus_text(macos_proto::FocusTextRequest {
        application: options.app,
        selector: Some(selector),
        expected_role: options.expected_role,
      })
      .await?
      .into_inner();
    ax_focus_result_from_proto(response)
  }
}

pub fn ax_focus_result_from_proto(response: macos_proto::FocusTextResponse) -> Result<auv_driver::AxFocusResult, tonic::Status> {
  let result = response.result.ok_or_else(|| tonic::Status::data_loss("FocusText response omitted AxFocusResult"))?;
  if result.app.trim().is_empty() || result.path.trim().is_empty() || result.role.trim().is_empty() {
    return Err(tonic::Status::data_loss("FocusText response omitted resolved AX identity"));
  }
  Ok(auv_driver::AxFocusResult {
    app: result.app,
    pid: result.pid,
    path: result.path,
    role: result.role,
    title: result.title,
    value: result.value,
    query: result.query,
    input_action_result: input_action_result_from_proto(
      result.action.ok_or_else(|| tonic::Status::data_loss("FocusText response omitted InputActionResult"))?,
    )?,
  })
}

fn input_action_result_from_proto(action: proto::InputActionResult) -> Result<auv_driver::InputActionResult, tonic::Status> {
  fn path(value: i32) -> Result<auv_driver::InputDeliveryPath, tonic::Status> {
    use proto::InputDeliveryPath as Wire;
    Ok(match Wire::try_from(value).map_err(|_| tonic::Status::data_loss("unknown InputDeliveryPath"))? {
      Wire::Unspecified => return Err(tonic::Status::data_loss("InputDeliveryPath was unspecified")),
      Wire::Noop => auv_driver::InputDeliveryPath::Noop,
      Wire::AxPress => auv_driver::InputDeliveryPath::AxPress,
      Wire::AxFocus => auv_driver::InputDeliveryPath::AxFocus,
      Wire::AxSetValue => auv_driver::InputDeliveryPath::AxSetValue,
      Wire::AxScroll => auv_driver::InputDeliveryPath::AxScroll,
      Wire::AxSelectedText => auv_driver::InputDeliveryPath::AxSelectedText,
      Wire::WindowTargetedMouse => auv_driver::InputDeliveryPath::WindowTargetedMouse,
      Wire::WindowTargetedWheel => auv_driver::InputDeliveryPath::WindowTargetedWheel,
      Wire::WindowTargetedKeyboard => auv_driver::InputDeliveryPath::WindowTargetedKeyboard,
      Wire::WindowTargetedKeyboardScroll => auv_driver::InputDeliveryPath::WindowTargetedKeyboardScroll,
      Wire::ClipboardPaste => auv_driver::InputDeliveryPath::ClipboardPaste,
      Wire::ForegroundSystemEvents => auv_driver::InputDeliveryPath::ForegroundSystemEvents,
      Wire::Unsupported => auv_driver::InputDeliveryPath::Unsupported,
    })
  }
  fn disturbance(value: i32) -> Result<auv_driver::DisturbanceLevel, tonic::Status> {
    use proto::DisturbanceLevel as Wire;
    Ok(match Wire::try_from(value).map_err(|_| tonic::Status::data_loss("unknown DisturbanceLevel"))? {
      Wire::Unspecified => return Err(tonic::Status::data_loss("DisturbanceLevel was unspecified")),
      Wire::None => auv_driver::DisturbanceLevel::None,
      Wire::Temporary => auv_driver::DisturbanceLevel::Temporary,
      Wire::Foreground => auv_driver::DisturbanceLevel::Foreground,
      Wire::Unknown => auv_driver::DisturbanceLevel::Unknown,
    })
  }

  let action = auv_driver::InputActionResult {
    selected_path: path(action.selected_path)?,
    attempts: action
      .attempts
      .into_iter()
      .map(|attempt| {
        Ok(auv_driver::InputAttempt {
          path: path(attempt.path)?,
          succeeded: attempt.succeeded,
          message: attempt.message,
        })
      })
      .collect::<Result<Vec<_>, tonic::Status>>()?,
    // TODO(input-action-result-wire-verification): the current protobuf shape
    // cannot carry semantic verification. Keep remote projections false until
    // an owner-approved producer/reader schema slice adds that evidence.
    verified: false,
    mouse_disturbance: disturbance(action.mouse_disturbance)?,
    focus_disturbance: disturbance(action.focus_disturbance)?,
    clipboard_disturbance: disturbance(action.clipboard_disturbance)?,
  };
  action.validate().map_err(|error| tonic::Status::data_loss(error.to_string()))?;
  Ok(action)
}

impl ApplicationClient {
  pub async fn activate_bundle_id(
    &self,
    bundle_id: impl Into<String>,
    settle: Option<prost_types::Duration>,
  ) -> Result<auv_driver::ApplicationActivationResult, tonic::Status> {
    let response = macos_proto::application_service_client::ApplicationServiceClient::new(self.runner.transport()?)
      .activate_bundle_id(macos_proto::ActivateBundleIdRequest {
        bundle_id: bundle_id.into(),
        settle,
      })
      .await?
      .into_inner();
    activation_result_from_proto(response)
  }
}

pub fn activation_result_from_proto(
  response: macos_proto::ActivateBundleIdResponse,
) -> Result<auv_driver::ApplicationActivationResult, tonic::Status> {
  use macos_proto::application_activation_verification::Verification;

  if response.requested_bundle_id.trim().is_empty() {
    return Err(tonic::Status::data_loss("ActivateBundleId response omitted requested_bundle_id"));
  }
  let verification = response
    .verification
    .and_then(|verification| verification.verification)
    .ok_or_else(|| tonic::Status::data_loss("ActivateBundleId response omitted verification"))?;
  let verification = match verification {
    Verification::VerifiedForeground(value) if !value.observed_bundle_id.trim().is_empty() => {
      auv_driver::ApplicationActivationVerification::VerifiedForeground {
        observed_bundle_id: value.observed_bundle_id,
      }
    }
    Verification::ForegroundMismatch(value) if !value.observed_bundle_id.trim().is_empty() => {
      auv_driver::ApplicationActivationVerification::ForegroundMismatch {
        observed_bundle_id: value.observed_bundle_id,
      }
    }
    Verification::Unavailable(value) if !value.reason.trim().is_empty() => auv_driver::ApplicationActivationVerification::Unavailable {
      reason: value.reason,
    },
    _ => return Err(tonic::Status::data_loss("ActivateBundleId response contained empty verification evidence")),
  };
  Ok(auv_driver::ApplicationActivationResult {
    requested_bundle_id: response.requested_bundle_id,
    verification,
  })
}

impl MediaControlClient {
  pub async fn now_playing(&self) -> Result<auv_media_macos::NowPlayingState, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .get_now_playing(macos_proto::GetNowPlayingRequest {})
      .await?
      .into_inner();
    now_playing_from_proto(response)
  }

  pub async fn play(&self) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .play(macos_proto::PlayRequest {})
      .await?
      .into_inner();
    media_control_outcome_from_proto(response.outcome, "play")
  }

  pub async fn pause(&self) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .pause(macos_proto::PauseRequest {})
      .await?
      .into_inner();
    media_control_outcome_from_proto(response.outcome, "pause")
  }

  pub async fn toggle_play_pause(&self) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .toggle_play_pause(macos_proto::TogglePlayPauseRequest {})
      .await?
      .into_inner();
    media_control_outcome_from_proto(response.outcome, "toggle")
  }

  pub async fn next_track(&self) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .next_track(macos_proto::NextTrackRequest {})
      .await?
      .into_inner();
    media_control_outcome_from_proto(response.outcome, "next")
  }

  pub async fn previous_track(&self) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
    let response = macos_proto::media_control_service_client::MediaControlServiceClient::new(self.runner.transport()?)
      .previous_track(macos_proto::PreviousTrackRequest {})
      .await?
      .into_inner();
    media_control_outcome_from_proto(response.outcome, "previous")
  }
}

pub fn now_playing_from_proto(response: macos_proto::GetNowPlayingResponse) -> Result<auv_media_macos::NowPlayingState, tonic::Status> {
  let state = response.state.ok_or_else(|| tonic::Status::data_loss("GetNowPlaying response omitted state"))?;
  now_playing_state_from_proto(state)
}

fn now_playing_state_from_proto(state: macos_proto::NowPlayingState) -> Result<auv_media_macos::NowPlayingState, tonic::Status> {
  for (field, value) in [
    ("duration_seconds", state.duration_seconds),
    ("elapsed_seconds", state.elapsed_seconds),
    ("playback_rate", state.playback_rate),
  ] {
    if value.is_some_and(|value| !value.is_finite()) {
      return Err(tonic::Status::data_loss(format!("GetNowPlaying returned non-finite {field}")));
    }
  }
  Ok(auv_media_macos::NowPlayingState {
    present: state.present,
    is_playing: state.is_playing,
    source_bundle_id: state.source_bundle_id,
    title: state.title,
    artist: state.artist,
    album: state.album,
    duration_seconds: state.duration_seconds,
    elapsed_seconds: state.elapsed_seconds,
    playback_rate: state.playback_rate,
    content_item_id: state.content_item_id,
    supports_like: state.supports_like,
    is_liked: state.is_liked,
  })
}

fn media_control_outcome_from_proto(
  outcome: Option<macos_proto::MediaControlOutcome>,
  command: &'static str,
) -> Result<auv_media_macos::output::MediaControlOutcome, tonic::Status> {
  let outcome = outcome.ok_or_else(|| tonic::Status::data_loss("media control response omitted outcome"))?;
  let before =
    now_playing_state_from_proto(outcome.before.ok_or_else(|| tonic::Status::data_loss("media control outcome omitted before state"))?)?;
  let after =
    now_playing_state_from_proto(outcome.after.ok_or_else(|| tonic::Status::data_loss("media control outcome omitted after state"))?)?;
  Ok(auv_media_macos::output::MediaControlOutcome {
    command,
    before: auv_media_macos::output::build_now_playing_output(&before),
    after: auv_media_macos::output::build_now_playing_output(&after),
    verified: outcome.verified,
  })
}

impl PermissionClient {
  pub async fn probe(&self) -> Result<auv_driver::PermissionProbe, tonic::Status> {
    let response = macos_proto::permission_service_client::PermissionServiceClient::new(self.runner.transport()?)
      .probe_permissions(macos_proto::ProbePermissionsRequest {})
      .await?
      .into_inner();
    permission_probe_from_proto(response)
  }
}

pub fn permission_probe_from_proto(response: macos_proto::ProbePermissionsResponse) -> Result<auv_driver::PermissionProbe, tonic::Status> {
  Ok(auv_driver::PermissionProbe {
    screen_recording: permission_status_from_proto(response.screen_recording, "screen_recording")?,
    screen_capture_kit: permission_status_from_proto(response.screen_capture_kit, "screen_capture_kit")?,
    accessibility: permission_status_from_proto(response.accessibility, "accessibility")?,
    automation_to_system_events: permission_status_from_proto(response.automation_to_system_events, "automation_to_system_events")?,
  })
}

fn permission_status_from_proto(value: i32, field: &'static str) -> Result<auv_driver::PermissionStatus, tonic::Status> {
  match macos_proto::PermissionStatus::try_from(value) {
    Ok(macos_proto::PermissionStatus::Granted) => Ok(auv_driver::PermissionStatus::Granted),
    Ok(macos_proto::PermissionStatus::Missing) => Ok(auv_driver::PermissionStatus::Missing),
    Ok(macos_proto::PermissionStatus::Unknown) => Ok(auv_driver::PermissionStatus::Unknown),
    Ok(macos_proto::PermissionStatus::Unspecified) | Err(_) => {
      Err(tonic::Status::data_loss(format!("ProbePermissions returned invalid {field} status")))
    }
  }
}

impl InputClient {
  pub async fn click_screen_point(
    &self,
    point: proto::ScreenPoint,
    options: Option<proto::ScreenClickOptions>,
  ) -> Result<proto::ClickScreenPointResponse, tonic::Status> {
    Ok(
      proto::input_service_client::InputServiceClient::new(self.runner.transport()?)
        .click_screen_point(proto::ClickScreenPointRequest {
          point: Some(point),
          options,
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn type_text(
    &self,
    text: impl Into<String>,
    options: Option<proto::TypeTextOptions>,
  ) -> Result<proto::TypeTextResponse, tonic::Status> {
    Ok(
      proto::input_service_client::InputServiceClient::new(self.runner.transport()?)
        .type_text(proto::TypeTextRequest {
          text: text.into(),
          options,
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn paste_text(
    &self,
    text: impl Into<String>,
    options: Option<proto::PasteTextOptions>,
  ) -> Result<proto::PasteTextResponse, tonic::Status> {
    Ok(
      proto::input_service_client::InputServiceClient::new(self.runner.transport()?)
        .paste_text(proto::PasteTextRequest {
          text: text.into(),
          options,
        })
        .await?
        .into_inner(),
    )
  }

  pub async fn press_key(
    &self,
    key: impl Into<String>,
    settle: Option<prost_types::Duration>,
  ) -> Result<proto::PressKeyResponse, tonic::Status> {
    Ok(
      proto::input_service_client::InputServiceClient::new(self.runner.transport()?)
        .press_key(proto::PressKeyRequest {
          key: key.into(),
          settle,
        })
        .await?
        .into_inner(),
    )
  }
}

#[derive(Clone, Debug, Default)]
pub struct FindTextOptions {
  pub region: Option<NormalizedRect>,
  pub custom_words: Vec<String>,
  pub recognition_languages: Vec<String>,
}

#[cfg(test)]
#[path = "runner_test.rs"]
mod tests;
