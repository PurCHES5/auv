use super::*;

fn disconnected_client() -> GrpcClient {
  let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:9").connect_lazy();
  GrpcClient::from_channel(channel)
}

fn route() -> auv_api_client::RunnerRoute {
  auv_api_client::RunnerRoute {
    device_id: Some("device_test".to_string()),
    run_id: Some("run_test".to_string()),
    runner_class: "auv.core.local".to_string(),
  }
}

#[tokio::test]
async fn runner_hierarchy_rejects_an_empty_class_before_any_transport_call() {
  let error = RunnerClient::new(
    disconnected_client(),
    auv_api_client::RunnerRoute {
      runner_class: String::new(),
      device_id: None,
      run_id: None,
    },
  )
  .expect_err("empty RunnerClass must fail");
  assert_eq!(error.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn resolved_window_child_retains_the_exact_resource_reference() {
  let runner = RunnerClient::new(disconnected_client(), route()).expect("runner client");
  let child = WindowClient {
    runner,
    window: proto::Window {
      r#ref: Some(proto::WindowRef {
        window_id: "window_test".to_string(),
      }),
      ..Default::default()
    },
    window_ref: proto::WindowRef {
      window_id: "window_test".to_string(),
    },
  };
  assert_eq!(child.reference().window_id, "window_test");
  assert_eq!(child.resource().r#ref.as_ref(), Some(child.reference()));
}

#[tokio::test]
async fn runner_input_exposes_typed_screen_point_click() {
  let runner = RunnerClient::new(disconnected_client(), route()).expect("runner client");
  let input = runner.input();
  let call = input.click_screen_point(proto::ScreenPoint { x: 10.0, y: 20.0 }, Some(Default::default()));
  drop(call);
}

#[test]
fn permission_mapper_preserves_explicit_statuses() {
  let probe = permission_probe_from_proto(macos_proto::ProbePermissionsResponse {
    screen_recording: macos_proto::PermissionStatus::Granted as i32,
    screen_capture_kit: macos_proto::PermissionStatus::Missing as i32,
    accessibility: macos_proto::PermissionStatus::Unknown as i32,
    automation_to_system_events: macos_proto::PermissionStatus::Granted as i32,
  })
  .expect("valid permission projection");
  assert_eq!(probe.screen_recording, auv_driver::PermissionStatus::Granted);
  assert_eq!(probe.screen_capture_kit, auv_driver::PermissionStatus::Missing);
  assert_eq!(probe.accessibility, auv_driver::PermissionStatus::Unknown);
  assert_eq!(probe.automation_to_system_events, auv_driver::PermissionStatus::Granted);
}

#[test]
fn permission_mapper_rejects_unspecified_and_unknown_wire_values() {
  for value in [macos_proto::PermissionStatus::Unspecified as i32, 99] {
    let error = permission_probe_from_proto(macos_proto::ProbePermissionsResponse {
      screen_recording: value,
      screen_capture_kit: macos_proto::PermissionStatus::Unknown as i32,
      accessibility: macos_proto::PermissionStatus::Unknown as i32,
      automation_to_system_events: macos_proto::PermissionStatus::Unknown as i32,
    })
    .expect_err("invalid wire status must not silently become Unknown");
    assert_eq!(error.code(), tonic::Code::DataLoss);
  }
}

#[test]
fn accessibility_mapper_preserves_ax_identity_and_delivery_evidence() {
  let result = ax_focus_result_from_proto(macos_proto::FocusTextResponse {
    result: Some(macos_proto::AxFocusResult {
      app: "com.example.Editor".to_string(),
      pid: 42,
      path: "root/AXTextArea[0]".to_string(),
      role: "AXTextArea".to_string(),
      title: "Document".to_string(),
      value: "draft".to_string(),
      // Exact-path selection intentionally has no query in the owner result.
      query: String::new(),
      action: Some(proto::InputActionResult {
        selected_path: proto::InputDeliveryPath::AxFocus as i32,
        attempts: vec![proto::InputAttempt {
          path: proto::InputDeliveryPath::AxFocus as i32,
          succeeded: true,
          message: None,
        }],
        mouse_disturbance: proto::DisturbanceLevel::None as i32,
        focus_disturbance: proto::DisturbanceLevel::Temporary as i32,
        clipboard_disturbance: proto::DisturbanceLevel::None as i32,
      }),
    }),
  })
  .expect("valid AX focus projection");

  assert_eq!(result.path, "root/AXTextArea[0]");
  assert!(result.query.is_empty());
  assert_eq!(result.input_action_result.selected_path, auv_driver::InputDeliveryPath::AxFocus);
}

#[test]
fn accessibility_mapper_rejects_missing_result_before_rendering() {
  let error = ax_focus_result_from_proto(macos_proto::FocusTextResponse::default()).expect_err("missing focus result");
  assert_eq!(error.code(), tonic::Code::DataLoss);
}

#[test]
fn application_activation_mapper_preserves_typed_verification() {
  use macos_proto::application_activation_verification::Verification;

  let result = activation_result_from_proto(macos_proto::ActivateBundleIdResponse {
    requested_bundle_id: "com.example.Requested".to_string(),
    verification: Some(macos_proto::ApplicationActivationVerification {
      verification: Some(Verification::ForegroundMismatch(macos_proto::ForegroundMismatch {
        observed_bundle_id: "com.example.Other".to_string(),
      })),
    }),
  })
  .expect("typed activation result");
  assert_eq!(result.requested_bundle_id, "com.example.Requested");
  assert_eq!(
    result.verification,
    auv_driver::ApplicationActivationVerification::ForegroundMismatch {
      observed_bundle_id: "com.example.Other".to_string(),
    }
  );
}

#[test]
fn application_activation_mapper_rejects_missing_or_empty_evidence() {
  let missing = activation_result_from_proto(macos_proto::ActivateBundleIdResponse {
    requested_bundle_id: "com.example.Requested".to_string(),
    verification: None,
  })
  .expect_err("missing verification must fail closed");
  assert_eq!(missing.code(), tonic::Code::DataLoss);

  let empty = activation_result_from_proto(macos_proto::ActivateBundleIdResponse {
    requested_bundle_id: "com.example.Requested".to_string(),
    verification: Some(macos_proto::ApplicationActivationVerification {
      verification: Some(macos_proto::application_activation_verification::Verification::Unavailable(
        macos_proto::VerificationUnavailable::default(),
      )),
    }),
  })
  .expect_err("empty reason must fail closed");
  assert_eq!(empty.code(), tonic::Code::DataLoss);
}

#[tokio::test]
async fn runner_exposes_hierarchical_macos_permission_client() {
  let runner = RunnerClient::new(disconnected_client(), route()).expect("runner client");
  let permissions = runner.macos().permissions();
  let call = permissions.probe();
  drop(call);
}

#[test]
fn now_playing_mapper_preserves_exact_owner_state() {
  let state = now_playing_from_proto(macos_proto::GetNowPlayingResponse {
    state: Some(macos_proto::NowPlayingState {
      present: true,
      is_playing: false,
      source_bundle_id: Some("com.apple.Music".to_string()),
      title: Some("Current Song".to_string()),
      artist: None,
      album: Some("Album".to_string()),
      duration_seconds: Some(245.5),
      elapsed_seconds: Some(61.25),
      playback_rate: Some(0.0),
      content_item_id: Some("track-42".to_string()),
      supports_like: None,
      is_liked: Some(false),
    }),
  })
  .expect("valid wire state");
  assert!(state.present);
  assert!(!state.is_playing);
  assert_eq!(state.source_bundle_id.as_deref(), Some("com.apple.Music"));
  assert_eq!(state.title.as_deref(), Some("Current Song"));
  assert_eq!(state.artist, None);
  assert_eq!(state.album.as_deref(), Some("Album"));
  assert_eq!(state.duration_seconds, Some(245.5));
  assert_eq!(state.elapsed_seconds, Some(61.25));
  assert_eq!(state.playback_rate, Some(0.0));
  assert_eq!(state.content_item_id.as_deref(), Some("track-42"));
  assert_eq!(state.supports_like, None);
  assert_eq!(state.is_liked, Some(false));
}

#[test]
fn now_playing_mapper_rejects_missing_or_non_finite_wire_state() {
  let missing = now_playing_from_proto(macos_proto::GetNowPlayingResponse::default()).expect_err("state is required");
  assert_eq!(missing.code(), tonic::Code::DataLoss);
  let invalid = now_playing_from_proto(macos_proto::GetNowPlayingResponse {
    state: Some(macos_proto::NowPlayingState {
      duration_seconds: Some(f64::NAN),
      ..Default::default()
    }),
  })
  .expect_err("non-finite wire value must fail closed");
  assert_eq!(invalid.code(), tonic::Code::DataLoss);
}

#[test]
fn media_control_mapper_preserves_owner_outcome_and_method_identity() {
  let state = macos_proto::NowPlayingState {
    present: true,
    is_playing: true,
    title: Some("Song".to_string()),
    playback_rate: Some(1.0),
    ..Default::default()
  };
  let outcome = media_control_outcome_from_proto(
    Some(macos_proto::MediaControlOutcome {
      before: Some(macos_proto::NowPlayingState {
        is_playing: false,
        playback_rate: Some(0.0),
        ..state.clone()
      }),
      after: Some(state),
      verified: true,
    }),
    "play",
  )
  .expect("valid outcome");
  assert_eq!(outcome.command, "play");
  assert!(!outcome.before.is_playing);
  assert!(outcome.after.is_playing);
  assert!(outcome.verified);
}

#[test]
fn media_control_mapper_rejects_missing_or_malformed_evidence() {
  assert_eq!(media_control_outcome_from_proto(None, "play").expect_err("outcome required").code(), tonic::Code::DataLoss);
  assert_eq!(
    media_control_outcome_from_proto(Some(macos_proto::MediaControlOutcome::default()), "play").expect_err("before required").code(),
    tonic::Code::DataLoss
  );
  let malformed = macos_proto::MediaControlOutcome {
    before: Some(macos_proto::NowPlayingState::default()),
    after: Some(macos_proto::NowPlayingState {
      elapsed_seconds: Some(f64::NAN),
      ..Default::default()
    }),
    verified: false,
  };
  assert_eq!(media_control_outcome_from_proto(Some(malformed), "next").expect_err("finite evidence required").code(), tonic::Code::DataLoss);
}

#[tokio::test]
async fn runner_exposes_hierarchical_macos_media_client() {
  let runner = RunnerClient::new(disconnected_client(), route()).expect("runner client");
  let media = runner.macos().media();
  drop(media.now_playing());
  drop(media.play());
  drop(media.pause());
  drop(media.toggle_play_pause());
  drop(media.next_track());
  drop(media.previous_track());
}
