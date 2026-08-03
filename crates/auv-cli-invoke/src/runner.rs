//! Typed invoke execution through a selected daemon-owned Runner.

async fn wait_for_selected_text<R, Call, Future, HasMatches>(
  command_id: &str,
  query: &str,
  options: auv_driver::WaitOptions,
  cancellation: &crate::InvokeCancellation,
  mut call: Call,
  has_matches: HasMatches,
) -> Result<R, String>
where
  Call: FnMut() -> Future,
  Future: std::future::Future<Output = Result<R, String>>,
  HasMatches: Fn(&R) -> bool,
{
  let started = std::time::Instant::now();
  loop {
    cancellation.check().map_err(|error| error.to_string())?;
    let response = call().await?;
    if has_matches(&response) {
      return Ok(response);
    }
    if started.elapsed() >= options.timeout {
      return Err(format!("{command_id} did not find text {query:?} before timeout"));
    }
    tokio::select! {
      _ = cancellation.cancelled() => return Err("invoke cancelled".to_string()),
      _ = tokio::time::sleep(options.poll_interval) => {}
    }
  }
}

pub async fn invoke(input: crate::InvokeCommandInput, context: auv::AuvContext) -> crate::InvokeCommandResult {
  let command_id = input.command_id.as_str();
  if command_id == "app.probePermissions" && input.target_application_id.is_some() {
    return Err("app.probePermissions cannot use --target".to_string());
  }
  if command_id == "app.activate" && input.target_application_id.as_ref().is_none_or(|target| target.trim().is_empty()) {
    return Err("app.activate requires --target".to_string());
  }
  if matches!(command_id, "input.focusText" | "input.axFocusText")
    && input.target_application_id.as_ref().is_none_or(|target| target.trim().is_empty())
  {
    return Err(format!("{command_id} requires --target"));
  }
  if command_id.starts_with("mediaControl.") && input.target_application_id.is_some() {
    return Err(if command_id == "mediaControl.nowPlaying" {
      "mediaControl.nowPlaying cannot use --target; the macOS now-playing state is system-wide".to_string()
    } else {
      format!("{command_id} cannot use --target; macOS media controls are system-wide")
    });
  }
  if command_id.starts_with("overlay.") && input.target_application_id.is_some() {
    return Err(format!("{command_id} cannot use --target; overlays use global screen coordinates"));
  }
  if command_id.starts_with("overlay.") {
    let plan = crate::commands::overlay::plan_overlay(&input)?;
    if input.dry_run || !input.overlay_enabled()? {
      return crate::commands::overlay::selected_overlay_output(&plan, false);
    }
  }
  if matches!(command_id, "input.typeText" | "input.pasteText" | "input.key") && input.target_application_id.is_some() {
    return Err(format!("{command_id} cannot use --target until typed input target activation is available"));
  }
  if matches!(command_id, "screen.findText" | "screen.waitForText" | "screen.clickText" | "screen.captureRegion")
    && input.target_application_id.is_some()
  {
    return Err(format!("{command_id} cannot use --target until typed target activation is available"));
  }
  let auv = auv::Client::from_context(context).await.map_err(|error| error.to_string())?;
  let run = auv.run(Default::default()).await.map_err(|error| format!("resolve selected Run failed: {error}"))?;
  let runner = run
    .runner(auv::client::RunnerOptions::default())
    .await
    .map_err(|error| format!("route core Runner for {command_id} failed: {error}"))?;

  let invoked = match command_id {
    "app.activate" => {
      let target = input.target_application_id.as_deref().expect("validated target").trim();
      runner
        .macos()
        .applications()
        .activate_bundle_id(
          target,
          Some(prost_types::Duration {
            seconds: 0,
            nanos: 150_000_000,
          }),
        )
        .await
        .map_err(|status| format!("ApplicationService/ActivateBundleId failed: {status}"))
        .and_then(|result| {
          if result.requested_bundle_id != target {
            return Err("ActivateBundleId response changed the requested bundle id".to_string());
          }
          crate::commands::app::activation_output(&result)
        })
    }
    "app.probePermissions" => runner
      .macos()
      .permissions()
      .probe()
      .await
      .map_err(|status| format!("PermissionService/ProbePermissions failed: {status}"))
      .and_then(|probe| crate::commands::app::permission_probe_output(&probe)),
    "input.focusText" | "input.axFocusText" => {
      let candidate = input.inputs.get("candidate").cloned().unwrap_or_default();
      let selector = if candidate.trim().is_empty() {
        auv_driver::AxTextSelector::Query(input.inputs.get("query").cloned().unwrap_or_default())
      } else {
        auv_driver::AxTextSelector::Path(candidate.clone())
      };
      runner
        .macos()
        .accessibility()
        .focus_text(auv_driver::FocusTextOptions {
          app: input.target_application_id.clone().expect("validated target"),
          selector,
          expected_role: None,
        })
        .await
        .map_err(|status| format!("AccessibilityService/FocusText failed: {status}"))
        .and_then(|result| crate::commands::input::focus_text_output(&result, &candidate))
    }
    "mediaControl.nowPlaying" => runner
      .macos()
      .media()
      .now_playing()
      .await
      .map_err(|status| format!("MediaControlService/GetNowPlaying failed: {status}"))
      .and_then(|state| crate::commands::media_control::now_playing_state_output(&state)),
    "mediaControl.play" => runner
      .macos()
      .media()
      .play()
      .await
      .map_err(|status| format!("MediaControlService/Play failed: {status}"))
      .and_then(|outcome| crate::commands::media_control::media_control_output(&outcome)),
    "mediaControl.pause" => runner
      .macos()
      .media()
      .pause()
      .await
      .map_err(|status| format!("MediaControlService/Pause failed: {status}"))
      .and_then(|outcome| crate::commands::media_control::media_control_output(&outcome)),
    "mediaControl.togglePlayPause" => runner
      .macos()
      .media()
      .toggle_play_pause()
      .await
      .map_err(|status| format!("MediaControlService/TogglePlayPause failed: {status}"))
      .and_then(|outcome| crate::commands::media_control::media_control_output(&outcome)),
    "mediaControl.next" => runner
      .macos()
      .media()
      .next_track()
      .await
      .map_err(|status| format!("MediaControlService/NextTrack failed: {status}"))
      .and_then(|outcome| crate::commands::media_control::media_control_output(&outcome)),
    "mediaControl.previous" => runner
      .macos()
      .media()
      .previous_track()
      .await
      .map_err(|status| format!("MediaControlService/PreviousTrack failed: {status}"))
      .and_then(|outcome| crate::commands::media_control::media_control_output(&outcome)),
    "overlay.outline" | "overlay.cursor" | "overlay.status" | "overlay.captureFrame" | "overlay.clickTarget" => {
      let plan = crate::commands::overlay::plan_overlay(&input)?;
      runner
        .overlay()
        .show(&plan.overlay, plan.options)
        .await
        .map_err(|status| format!("OverlayService/ShowOverlay failed: {status}"))
        .and_then(|()| crate::commands::overlay::selected_overlay_output(&plan, true))
    }
    "display.list" => {
      runner.displays().list().await.map_err(|status| format!("DisplayService/ListDisplays failed: {status}")).and_then(|displays| {
        let displays = displays.into_iter().map(display_from_proto).collect::<Result<Vec<_>, String>>()?;
        crate::commands::display::list_displays_output(&auv_driver::ObservedDisplays { displays })
      })
    }
    "display.capture" => match runner.displays().capture(None).await {
      Err(status) => Err(format!("CaptureService/CaptureDisplay failed: {status}")),
      Ok(response) => {
        let capture = (|| {
          Ok(auv_driver::DisplayCapture {
            display: display_from_proto(response.display.ok_or_else(|| "CaptureDisplay response omitted Display".to_string())?)?,
            capture: capture_from_proto(response.capture.ok_or_else(|| "CaptureDisplay response omitted CapturedFrame".to_string())?)?,
          })
        })();
        match capture {
          Ok(capture) => crate::commands::display::recorded_display_capture_output(&capture).await,
          Err(error) => Err(error),
        }
      }
    },
    "screen.captureRegion" => match selected_screen_region(&input) {
      Err(error) => Err(error),
      Ok(region) => match runner.displays().capture_region(region, None).await {
        Err(status) => Err(format!("CaptureService/CaptureRegion failed: {status}")),
        Ok(response) => {
          let capture = (|| {
            Ok(auv_driver::RegionCapture {
              display: display_from_proto(response.display.ok_or_else(|| "CaptureRegion response omitted Display".to_string())?)?,
              capture: capture_from_proto(response.capture.ok_or_else(|| "CaptureRegion response omitted CapturedFrame".to_string())?)?,
            })
          })();
          match capture {
            Ok(capture) => crate::commands::screen::recorded_region_capture_output(&capture).await,
            Err(error) => Err(error),
          }
        }
      },
    },
    "window.list" => {
      runner.windows().list().await.map_err(|status| format!("WindowService/ListWindows failed: {status}")).and_then(|windows| {
        let windows = windows.into_iter().map(window_from_proto).collect::<Result<Vec<_>, String>>()?;
        crate::commands::window::list_windows_output(&windows)
      })
    }
    "window.capture" => {
      let selector = selected_window_selector(&input);
      let response = match runner.windows().resolve(selector).await {
        Err(status) => Err(status),
        Ok(window) => window.capture().await,
      };
      match response {
        Err(status) => Err(format!("WindowService/ResolveWindow or CaptureService/CaptureWindow failed: {status}")),
        Ok(response) => {
          let capture = (|| {
            Ok(crate::commands::window::WindowCapture {
              window: window_from_proto(response.window.ok_or_else(|| "CaptureWindow response omitted Window".to_string())?)?,
              capture: capture_from_proto(response.capture.ok_or_else(|| "CaptureWindow response omitted CapturedFrame".to_string())?)?,
            })
          })();
          match capture {
            Ok(capture) => crate::commands::window::recorded_window_capture_output(&capture).await,
            Err(error) => Err(error),
          }
        }
      }
    }
    "window.findText" => match input.inputs.get("query").cloned() {
      None => Err("window.findText omitted its typed query argument".to_string()),
      Some(query) => {
        let response = match runner.windows().resolve(selected_window_selector(&input)).await {
          Err(status) => Err(status),
          Ok(window) => window.find_text(query).await,
        };
        match response {
          Err(status) => Err(format!("WindowService/ResolveWindow or TextRecognitionService/FindWindowText failed: {status}")),
          Ok(response) => {
            let projected = (|| {
              let result = crate::commands::window::WindowTextRecognition {
                window: window_from_proto(response.window.ok_or_else(|| "FindWindowText response omitted Window".to_string())?)?,
                matches: ocr_matches_from_proto(response.matches)?,
              };
              let capture =
                capture_from_proto(response.capture.ok_or_else(|| "FindWindowText response omitted source capture".to_string())?)?;
              Ok((result, capture))
            })();
            match projected {
              Ok((result, capture)) => crate::commands::window::recorded_window_text_matches_output(&result, &capture),
              Err(error) => Err(error),
            }
          }
        }
      }
    },
    "window.waitForText" => match input.inputs.get("query").cloned() {
      None => Err("window.waitForText omitted its typed query argument".to_string()),
      Some(query) => match runner.windows().resolve(selected_window_selector(&input)).await {
        Err(status) => Err(format!("WindowService/ResolveWindow failed: {status}")),
        Ok(window) => {
          let response = wait_for_selected_text(
            command_id,
            &query,
            auv_driver::WaitOptions::default(),
            &input.cancellation,
            || {
              let window = window.clone();
              let query = query.clone();
              async move { window.find_text(query).await.map_err(|status| format!("TextRecognitionService/FindWindowText failed: {status}")) }
            },
            |response| !response.matches.is_empty(),
          )
          .await;
          match response {
            Err(error) => Err(error),
            Ok(response) => {
              let projected = (|| {
                let result = crate::commands::window::WindowTextRecognition {
                  window: window_from_proto(response.window.ok_or_else(|| "FindWindowText response omitted Window".to_string())?)?,
                  matches: ocr_matches_from_proto(response.matches)?,
                };
                let capture =
                  capture_from_proto(response.capture.ok_or_else(|| "FindWindowText response omitted source capture".to_string())?)?;
                Ok((result, capture))
              })();
              match projected {
                Ok((result, capture)) => crate::commands::window::recorded_window_text_matches_output(&result, &capture),
                Err(error) => Err(error),
              }
            }
          }
        }
      },
    },
    "screen.findText" => match input.inputs.get("query").cloned() {
      None => Err("screen.findText omitted its typed query argument".to_string()),
      Some(query) => {
        let response = runner.displays().find_text(None, query).await;
        match response {
          Err(status) => Err(format!("TextRecognitionService/FindDisplayText failed: {status}")),
          Ok(response) => {
            let projected = (|| {
              let _display = display_from_proto(response.display.ok_or_else(|| "FindDisplayText response omitted Display".to_string())?)?;
              let matches = ocr_matches_from_proto(response.matches)?;
              let capture =
                capture_from_proto(response.capture.ok_or_else(|| "FindDisplayText response omitted source capture".to_string())?)?;
              Ok((matches, capture))
            })();
            match projected {
              Ok((matches, capture)) => crate::commands::screen::recorded_screen_text_matches_output(&matches, &capture),
              Err(error) => Err(error),
            }
          }
        }
      }
    },
    "screen.clickText" => {
      async {
        let query = input.inputs.get("query").cloned().ok_or_else(|| "screen.clickText omitted its typed query argument".to_string())?;
        let recognized = runner
          .displays()
          .find_text(None, query.clone())
          .await
          .map_err(|status| format!("TextRecognitionService/FindDisplayText failed: {status}"))?;
        let _display = display_from_proto(recognized.display.ok_or_else(|| "FindDisplayText response omitted Display".to_string())?)?;
        let matches = ocr_matches_from_proto(recognized.matches)?;
        let capture = capture_from_proto(recognized.capture.ok_or_else(|| "FindDisplayText response omitted source capture".to_string())?)?;
        let point = matches.best_match().ok_or_else(|| format!("screen.clickText did not find text {query:?}"))?.action_point();
        let click = selected_click_options(&input)?.click;
        let response = runner
          .input()
          .click_screen_point(
            auv_api_proto::auv::api::driver::v1::ScreenPoint {
              x: point.x,
              y: point.y,
            },
            Some(auv_api_proto::auv::api::driver::v1::ScreenClickOptions { click }),
          )
          .await
          .map_err(|status| format!("InputService/ClickScreenPoint failed: {status}"))?;
        let point = response.point.ok_or_else(|| "ClickScreenPoint response omitted ScreenPoint".to_string())?;
        let action =
          input_action_from_proto(response.action.ok_or_else(|| "ClickScreenPoint response omitted InputActionResult".to_string())?)?;
        let result = crate::commands::screen::ScreenTextClick {
          matches,
          point: auv_driver::Point::new(point.x, point.y),
          action,
        };
        crate::commands::screen::recorded_screen_text_click_output(&result, &capture)
      }
      .await
    }
    "screen.waitForText" => match input.inputs.get("query").cloned() {
      None => Err("screen.waitForText omitted its typed query argument".to_string()),
      Some(query) => {
        let displays = runner.displays();
        let response = wait_for_selected_text(
          command_id,
          &query,
          auv_driver::WaitOptions::default(),
          &input.cancellation,
          || {
            let displays = displays.clone();
            let query = query.clone();
            async move {
              displays.find_text(None, query).await.map_err(|status| format!("TextRecognitionService/FindDisplayText failed: {status}"))
            }
          },
          |response| !response.matches.is_empty(),
        )
        .await;
        match response {
          Err(error) => Err(error),
          Ok(response) => {
            let projected = (|| {
              let _display = display_from_proto(response.display.ok_or_else(|| "FindDisplayText response omitted Display".to_string())?)?;
              let matches = ocr_matches_from_proto(response.matches)?;
              let capture =
                capture_from_proto(response.capture.ok_or_else(|| "FindDisplayText response omitted source capture".to_string())?)?;
              Ok((matches, capture))
            })();
            match projected {
              Ok((matches, capture)) => crate::commands::screen::recorded_screen_text_matches_output(&matches, &capture),
              Err(error) => Err(error),
            }
          }
        }
      }
    },
    "window.clickText" => {
      async {
        let query = input.inputs.get("query").cloned().ok_or_else(|| "window.clickText omitted its typed query argument".to_string())?;
        let selected_index = input
          .inputs
          .get("index")
          .map(|value| value.parse::<usize>().map_err(|error| format!("window.clickText has invalid --index: {error}")))
          .transpose()?
          .unwrap_or(0);
        let resolved = runner
          .windows()
          .resolve(selected_window_selector(&input))
          .await
          .map_err(|status| format!("WindowService/ResolveWindow failed: {status}"))?;
        let resolved_window = window_from_proto(resolved.resource().clone())?;
        let recognized =
          resolved.find_text(query.clone()).await.map_err(|status| format!("TextRecognitionService/FindWindowText failed: {status}"))?;
        let matches = ocr_matches_from_proto(recognized.matches)?;
        let capture = capture_from_proto(recognized.capture.ok_or_else(|| "FindWindowText response omitted source capture".to_string())?)?;
        let matched = crate::commands::window::selected_window_text_match(&matches, &query, selected_index)?;
        let point = matched_window_point(&resolved_window, matched)?;
        let wire_options = selected_click_options(&input)?;
        let options = driver_click_options_from_proto(&wire_options)?;
        let response = resolved
          .click(
            auv_api_proto::auv::api::driver::v1::WindowPoint {
              x: point.point().x,
              y: point.point().y,
            },
            Some(wire_options),
          )
          .await
          .map_err(|status| format!("InputService/ClickWindowPoint failed: {status}"))?;
        let clicked_window = window_from_proto(response.window.ok_or_else(|| "ClickWindowPoint response omitted Window".to_string())?)?;
        if clicked_window.reference != resolved_window.reference {
          return Err("ClickWindowPoint response changed the resolved WindowRef".to_string());
        }
        let returned_point = response.point.ok_or_else(|| "ClickWindowPoint response omitted WindowPoint".to_string())?;
        let action =
          input_action_from_proto(response.action.ok_or_else(|| "ClickWindowPoint response omitted InputActionResult".to_string())?)?;
        let result = crate::commands::window::WindowTextClick {
          window: clicked_window,
          matches,
          selected_index,
          point: auv_driver::WindowPoint::new(returned_point.x, returned_point.y),
          options,
          action,
        };
        crate::commands::window::recorded_window_text_click_output(&result, &capture)
      }
      .await
    }
    "input.typeText" => match input.inputs.get("text").cloned() {
      None => Err("input.typeText omitted its typed text argument".to_string()),
      Some(text) => runner
        .input()
        .type_text(text, Some(Default::default()))
        .await
        .map_err(|status| format!("InputService/TypeText failed: {status}"))
        .and_then(|response| {
          let action = input_action_from_proto(response.action.ok_or_else(|| "TypeText response omitted InputActionResult".to_string())?)?;
          crate::emit_input_action_result(&action);
          crate::commands::input::input_action_output(&action)
        }),
    },
    "input.pasteText" => match input.inputs.get("text").cloned() {
      None => Err("input.pasteText omitted its typed text argument".to_string()),
      Some(text) => runner
        .input()
        .paste_text(text, Some(Default::default()))
        .await
        .map_err(|status| format!("InputService/PasteText failed: {status}"))
        .and_then(|response| {
          let action = input_action_from_proto(response.action.ok_or_else(|| "PasteText response omitted InputActionResult".to_string())?)?;
          crate::emit_input_action_result(&action);
          crate::commands::input::input_action_output(&action)
        }),
    },
    "input.key" => match input.inputs.get("key").cloned() {
      None => Err("input.key omitted its typed key argument".to_string()),
      Some(key) => {
        runner.input().press_key(key.clone(), None).await.map_err(|status| format!("InputService/PressKey failed: {status}")).and_then(
          |response| {
            let action = input_action_from_proto(response.action.ok_or_else(|| "PressKey response omitted InputActionResult".to_string())?)?;
            crate::emit_input_action_result(&action);
            crate::commands::input::press_key_output(&action, &key)
          },
        )
      }
    },
    "input.clickWindowPoint" => match runner.windows().resolve(selected_window_selector(&input)).await {
      Err(status) => Err(format!("WindowService/ResolveWindow failed: {status}")),
      Ok(resolved) => match window_from_proto(resolved.resource().clone()) {
        Err(error) => Err(error),
        Ok(window) => match (selected_window_point(&input, &window), selected_click_options(&input)) {
          (Err(error), _) | (_, Err(error)) => Err(error),
          (Ok(point), Ok(options)) => {
            match resolved
              .click(
                auv_api_proto::auv::api::driver::v1::WindowPoint {
                  x: point.point().x,
                  y: point.point().y,
                },
                Some(options),
              )
              .await
            {
              Err(status) => Err(format!("InputService/ClickWindowPoint failed: {status}")),
              Ok(response) => {
                let projected = (|| {
                  let point = response.point.ok_or_else(|| "ClickWindowPoint response omitted WindowPoint".to_string())?;
                  let action = input_action_from_proto(
                    response.action.ok_or_else(|| "ClickWindowPoint response omitted InputActionResult".to_string())?,
                  )?;
                  Ok(crate::commands::input::WindowPointClickResult {
                    window: window_from_proto(response.window.ok_or_else(|| "ClickWindowPoint response omitted Window".to_string())?)?,
                    point: auv_driver::WindowPoint::new(point.x, point.y),
                    action: Some(action),
                  })
                })();
                match projected {
                  Ok(result) => {
                    if let Some(action) = result.action.as_ref() {
                      crate::emit_input_action_result(action);
                    }
                    crate::commands::input::window_point_click_output_without_overlay(result)
                  }
                  Err(error) => Err(error),
                }
              }
            }
          }
        },
      },
    },
    _ => unreachable!("typed Runner adapter was selected above"),
  };
  invoked
}

fn display_from_proto(display: auv_api_proto::auv::api::driver::v1::Display) -> Result<auv_driver::Display, String> {
  let frame = display.frame.ok_or_else(|| format!("Display {:?} omitted its screen frame", display.display_id))?;
  Ok(auv_driver::Display {
    id: display.display_id,
    name: display.name,
    frame: auv_driver::Rect::new(frame.x, frame.y, frame.width, frame.height),
    coordinate_space: auv_driver::CoordinateSpace::Screen,
    scale_factor: display.scale_factor,
    is_primary: display.primary,
    is_builtin: display.builtin,
  })
}

fn window_from_proto(window: auv_api_proto::auv::api::driver::v1::Window) -> Result<auv_driver::Window, String> {
  let reference = window.r#ref.ok_or_else(|| "Window omitted its reference".to_string())?;
  let frame = window.frame.ok_or_else(|| format!("Window {:?} omitted its screen frame", reference.window_id))?;
  Ok(auv_driver::Window {
    reference: auv_driver::WindowRef {
      id: reference.window_id,
    },
    title: window.title,
    app_name: window.application_name,
    app_bundle_id: window.application_bundle_id,
    process_id: window.process_id,
    frame: auv_driver::Rect::new(frame.x, frame.y, frame.width, frame.height),
    coordinate_space: auv_driver::CoordinateSpace::Screen,
    is_main: window.is_main,
    is_visible: window.is_visible,
  })
}

fn capture_from_proto(capture: auv_api_proto::auv::api::driver::v1::CapturedFrame) -> Result<auv_driver::Capture, String> {
  let image = capture.image.ok_or_else(|| "CapturedFrame omitted its RGBA image".to_string())?;
  let bounds = capture.bounds.ok_or_else(|| "CapturedFrame omitted its screen bounds".to_string())?;
  let image = image::RgbaImage::from_raw(image.width, image.height, image.data)
    .ok_or_else(|| "CapturedFrame contains malformed RGBA8 data".to_string())?;
  Ok(auv_driver::Capture {
    image,
    bounds: auv_driver::Rect::new(bounds.x, bounds.y, bounds.width, bounds.height),
    scale_factor: capture.scale_factor,
    backend: capture.backend,
    fallback_reason: capture.fallback_reason,
  })
}

fn ocr_matches_from_proto(matches: Vec<auv_api_proto::auv::api::driver::v1::TextMatch>) -> Result<auv_driver::OcrMatches, String> {
  Ok(auv_driver::OcrMatches {
    matches: matches
      .into_iter()
      .map(|matched| {
        let bounds = matched.bounds.ok_or_else(|| format!("text match {:?} omitted its screen bounds", matched.text))?;
        Ok(auv_driver::OcrMatch {
          text: matched.text,
          confidence: matched.confidence,
          bounds: auv_driver::Rect::new(bounds.x, bounds.y, bounds.width, bounds.height),
        })
      })
      .collect::<Result<Vec<_>, String>>()?,
  })
}

fn selected_screen_region(input: &crate::InvokeCommandInput) -> Result<auv_api_proto::auv::api::driver::v1::ScreenRect, String> {
  let number = |name: &str| {
    input
      .inputs
      .get(name)
      .ok_or_else(|| format!("screen.captureRegion omitted --{name}"))?
      .parse::<f64>()
      .map_err(|error| format!("screen.captureRegion has invalid --{name}: {error}"))
  };
  let region = auv_api_proto::auv::api::driver::v1::ScreenRect {
    x: number("x")?,
    y: number("y")?,
    width: number("width")?,
    height: number("height")?,
  };
  if !region.x.is_finite() || !region.y.is_finite() {
    return Err("screen.captureRegion requires finite --x and --y".to_string());
  }
  if !region.width.is_finite() || !region.height.is_finite() || region.width <= 0.0 || region.height <= 0.0 {
    return Err("screen.captureRegion requires --width and --height greater than zero".to_string());
  }
  Ok(region)
}

fn selected_window_point(input: &crate::InvokeCommandInput, window: &auv_driver::Window) -> Result<auv_driver::WindowPoint, String> {
  let number = |name: &str| {
    input
      .inputs
      .get(name)
      .map(|value| value.parse::<f64>().map_err(|error| format!("input.clickWindowPoint has invalid --{name}: {error}")))
      .transpose()
  };
  let offset_x = number("offset-x")?;
  let offset_y = number("offset-y")?;
  let relative_x = number("relative-x")?;
  let relative_y = number("relative-y")?;
  let point = match (offset_x, offset_y, relative_x, relative_y) {
    (Some(x), Some(y), None, None) if x.is_finite() && y.is_finite() && x >= 0.0 && y >= 0.0 => auv_driver::WindowPoint::new(x, y),
    (None, None, Some(x), Some(y)) if x.is_finite() && y.is_finite() && (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) => {
      auv_driver::WindowPoint::new(window.frame.size.width * x, window.frame.size.height * y)
    }
    (Some(_), Some(_), None, None) => return Err("input.clickWindowPoint requires finite non-negative window offsets".to_string()),
    (None, None, Some(_), Some(_)) => return Err("input.clickWindowPoint requires relative coordinates within 0..=1".to_string()),
    _ => return Err("input.clickWindowPoint requires --offset-x/--offset-y or --relative-x/--relative-y".to_string()),
  };
  let point_value = point.point();
  if !(0.0..=window.frame.size.width).contains(&point_value.x) || !(0.0..=window.frame.size.height).contains(&point_value.y) {
    return Err(format!(
      "input.clickWindowPoint point {},{} is outside target window bounds 0..={},0..={}",
      point_value.x, point_value.y, window.frame.size.width, window.frame.size.height
    ));
  }
  Ok(point)
}

fn selected_click_options(input: &crate::InvokeCommandInput) -> Result<auv_api_proto::auv::api::driver::v1::ClickOptions, String> {
  use auv_api_proto::auv::api::driver::v1::{InputPolicy, WindowClickStrategy};

  let command_id = input.command_id.as_str();

  let policy = match input.inputs.get("input-policy").map(String::as_str) {
    None if command_id == "screen.clickText" => InputPolicy::ForegroundPreferred,
    None | Some("background-preferred") => InputPolicy::BackgroundPreferred,
    Some("background-only") => InputPolicy::BackgroundOnly,
    Some("foreground-preferred") => InputPolicy::ForegroundPreferred,
    Some(value) => return Err(format!("{command_id} has unknown --input-policy {value:?}")),
  };
  let count = input
    .inputs
    .get("click-count")
    .map(|value| value.parse::<u32>().map_err(|error| format!("{command_id} has invalid --click-count: {error}")))
    .transpose()?
    .unwrap_or(1);
  if !(1..=u32::from(u8::MAX)).contains(&count) {
    return Err(format!("{command_id} requires --click-count within 1..=255"));
  }
  let interval_ms = input
    .inputs
    .get("click-interval-ms")
    .map(|value| value.parse::<u64>().map_err(|error| format!("{command_id} has invalid --click-interval-ms: {error}")))
    .transpose()?
    .unwrap_or(75);
  let interval = (count > 1).then(|| prost_types::Duration {
    seconds: i64::try_from(interval_ms / 1000).unwrap_or(i64::MAX),
    nanos: i32::try_from((interval_ms % 1000) * 1_000_000).expect("subsecond milliseconds fit i32"),
  });
  if count > 1 && interval_ms == 0 {
    return Err(format!("{command_id} requires a positive --click-interval-ms for repeated clicks"));
  }
  Ok(auv_api_proto::auv::api::driver::v1::ClickOptions {
    policy: policy as i32,
    click: Some(auv_api_proto::auv::api::driver::v1::Click { count, interval }),
    window_strategy: WindowClickStrategy::ChromiumCompatible as i32,
  })
}

fn driver_click_options_from_proto(options: &auv_api_proto::auv::api::driver::v1::ClickOptions) -> Result<auv_driver::ClickOptions, String> {
  use auv_api_proto::auv::api::driver::v1::{InputPolicy as ProtoPolicy, WindowClickStrategy as ProtoStrategy};

  let policy = match ProtoPolicy::try_from(options.policy).map_err(|_| format!("unknown InputPolicy value {}", options.policy))? {
    ProtoPolicy::Unspecified | ProtoPolicy::BackgroundPreferred => auv_driver::InputPolicy::BackgroundPreferred,
    ProtoPolicy::BackgroundOnly => auv_driver::InputPolicy::BackgroundOnly,
    ProtoPolicy::ForegroundPreferred => auv_driver::InputPolicy::ForegroundPreferred,
  };
  let click = match options.click.as_ref() {
    None => auv_driver::Click::Single,
    Some(click) if click.count == 1 => auv_driver::Click::Single,
    Some(click) => {
      let interval = click.interval.as_ref().ok_or_else(|| "repeated click omitted its interval".to_string())?;
      if interval.seconds < 0 || interval.nanos < 0 {
        return Err("click interval must not be negative".to_string());
      }
      let interval = std::time::Duration::new(
        u64::try_from(interval.seconds).map_err(|_| "click interval seconds do not fit u64".to_string())?,
        u32::try_from(interval.nanos).map_err(|_| "click interval nanos do not fit u32".to_string())?,
      );
      match click.count {
        2 => auv_driver::Click::Double { interval },
        count if (3..=u32::from(u8::MAX)).contains(&count) => auv_driver::Click::Repeated {
          count: u8::try_from(count).expect("validated click count fits u8"),
          interval,
        },
        count => return Err(format!("click count {count} is outside 1..=255")),
      }
    }
  };
  let window_strategy = match ProtoStrategy::try_from(options.window_strategy) {
    Ok(ProtoStrategy::Unspecified | ProtoStrategy::ChromiumCompatible) => auv_driver::WindowClickStrategy::ChromiumCompatible,
    Ok(ProtoStrategy::PidTargeted) => auv_driver::WindowClickStrategy::PidTargeted,
    Err(_) => return Err(format!("unknown WindowClickStrategy value {}", options.window_strategy)),
  };
  Ok(auv_driver::ClickOptions {
    policy,
    click,
    window_strategy,
  })
}

fn matched_window_point(window: &auv_driver::Window, matched: &auv_driver::OcrMatch) -> Result<auv_driver::WindowPoint, String> {
  let screen_point = matched.action_point();
  let point = auv_driver::WindowPoint::new(screen_point.x - window.frame.origin.x, screen_point.y - window.frame.origin.y);
  let point_value = point.point();
  if !(0.0..=window.frame.size.width).contains(&point_value.x) || !(0.0..=window.frame.size.height).contains(&point_value.y) {
    return Err(format!("recognized text point {},{} is outside resolved window bounds", screen_point.x, screen_point.y));
  }
  Ok(point)
}

fn selected_window_selector(input: &crate::InvokeCommandInput) -> auv_api_proto::auv::api::driver::v1::WindowSelector {
  use auv_api_proto::auv::api::driver::v1::window_selector::{Application, Window};

  let application = input
    .target_application_id
    .as_ref()
    // TODO(cross-platform-application-selector): `--target` currently carries
    // an application id and therefore maps to bundle/accessibility id. Add an
    // explicit application-name selector when the CLI contract can distinguish
    // ids from names; do not guess from punctuation or silently retry.
    .map(|bundle_id| Application::ApplicationBundleId(bundle_id.clone()))
    .unwrap_or(Application::FrontmostApplication(true));
  let window = input
    .inputs
    .get("title")
    .filter(|title| !title.trim().is_empty())
    .map(|title| Window::TitleContains(title.clone()))
    .unwrap_or(Window::MainVisible(true));
  auv_api_proto::auv::api::driver::v1::WindowSelector {
    application: Some(application),
    window: Some(window),
  }
}

fn input_action_from_proto(action: auv_api_proto::auv::api::driver::v1::InputActionResult) -> Result<auv_driver::InputActionResult, String> {
  use auv_api_proto::auv::api::driver::v1::{DisturbanceLevel as ProtoDisturbance, InputDeliveryPath as ProtoPath};

  fn path(value: i32) -> Result<auv_driver::InputDeliveryPath, String> {
    Ok(match ProtoPath::try_from(value).map_err(|_| format!("unknown InputDeliveryPath value {value}"))? {
      ProtoPath::Unspecified => return Err("InputDeliveryPath must not be unspecified".to_string()),
      ProtoPath::Noop => auv_driver::InputDeliveryPath::Noop,
      ProtoPath::AxPress => auv_driver::InputDeliveryPath::AxPress,
      ProtoPath::AxFocus => auv_driver::InputDeliveryPath::AxFocus,
      ProtoPath::AxSetValue => auv_driver::InputDeliveryPath::AxSetValue,
      ProtoPath::AxScroll => auv_driver::InputDeliveryPath::AxScroll,
      ProtoPath::AxSelectedText => auv_driver::InputDeliveryPath::AxSelectedText,
      ProtoPath::WindowTargetedMouse => auv_driver::InputDeliveryPath::WindowTargetedMouse,
      ProtoPath::WindowTargetedWheel => auv_driver::InputDeliveryPath::WindowTargetedWheel,
      ProtoPath::WindowTargetedKeyboard => auv_driver::InputDeliveryPath::WindowTargetedKeyboard,
      ProtoPath::WindowTargetedKeyboardScroll => auv_driver::InputDeliveryPath::WindowTargetedKeyboardScroll,
      ProtoPath::ClipboardPaste => auv_driver::InputDeliveryPath::ClipboardPaste,
      ProtoPath::ForegroundSystemEvents => auv_driver::InputDeliveryPath::ForegroundSystemEvents,
      ProtoPath::Unsupported => auv_driver::InputDeliveryPath::Unsupported,
    })
  }

  fn disturbance(value: i32) -> Result<auv_driver::DisturbanceLevel, String> {
    Ok(match ProtoDisturbance::try_from(value).map_err(|_| format!("unknown DisturbanceLevel value {value}"))? {
      ProtoDisturbance::Unspecified => return Err("DisturbanceLevel must not be unspecified".to_string()),
      ProtoDisturbance::None => auv_driver::DisturbanceLevel::None,
      ProtoDisturbance::Temporary => auv_driver::DisturbanceLevel::Temporary,
      ProtoDisturbance::Foreground => auv_driver::DisturbanceLevel::Foreground,
      ProtoDisturbance::Unknown => auv_driver::DisturbanceLevel::Unknown,
    })
  }

  let result = auv_driver::InputActionResult {
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
      .collect::<Result<Vec<_>, String>>()?,
    // TODO(input-action-result-wire-verification): the current protobuf shape
    // cannot carry semantic verification. Keep remote projections false until
    // an owner-approved producer/reader schema slice adds that evidence.
    verified: false,
    mouse_disturbance: disturbance(action.mouse_disturbance)?,
    focus_disturbance: disturbance(action.focus_disturbance)?,
    clipboard_disturbance: disturbance(action.clipboard_disturbance)?,
  };
  result.validate()?;
  Ok(result)
}
#[cfg(test)]
#[path = "runner_test.rs"]
mod tests;
