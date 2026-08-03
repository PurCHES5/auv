use super::*;

#[test]
fn invoke_preserves_root_device_and_run_selection() {
  let command = parse_cli(&[
    "--device-id".to_string(),
    "device_local".to_string(),
    "--run".to_string(),
    "run_parent".to_string(),
    "invoke".to_string(),
    "display.list".to_string(),
    "--json".to_string(),
  ])
  .expect("parse selected invoke command");

  let CliCommand::Invoke { parent_context, .. } = command else {
    panic!("expected invoke command");
  };
  assert_eq!(parent_context.device_id.as_deref(), Some("device_local"));
  assert_eq!(parent_context.run_id.as_deref(), Some("run_parent"));
}

#[test]
fn serve_accepts_repeated_trusted_runner_provider_manifests() {
  let command = parse_cli(&[
    "serve".to_string(),
    "--runner-provider".to_string(),
    "first.json".to_string(),
    "--runner-provider".to_string(),
    "second.json".to_string(),
  ])
  .expect("parse Runner provider manifests");

  match command {
    CliCommand::Serve {
      runner_providers, ..
    } => {
      assert_eq!(runner_providers, vec![PathBuf::from("first.json"), PathBuf::from("second.json")]);
    }
    command => panic!("unexpected command: {command:?}"),
  }
}

#[test]
fn serve_accepts_repeated_local_and_paired_bearer_listeners() {
  let command = parse_cli(&[
    "serve".to_string(),
    "--listen".to_string(),
    "unix:///tmp/auv.sock".to_string(),
    "--listen".to_string(),
    "http://0.0.0.0:9847".to_string(),
    "--pairing-store".to_string(),
    "pairings.json".to_string(),
  ])
  .expect("parse local and paired listeners");

  let CliCommand::Serve {
    listeners,
    pairing_store,
    ..
  } = command
  else {
    panic!("expected serve command");
  };
  assert_eq!(listeners, ["unix:///tmp/auv.sock", "http://0.0.0.0:9847"]);
  assert_eq!(pairing_store, Some(PathBuf::from("pairings.json")));
}

#[test]
fn api_server_remote_listener_requires_pairing_store() {
  let error = parse_cli(&[
    "api-server".to_string(),
    "serve".to_string(),
    "--remote-listen".to_string(),
    "0.0.0.0".to_string(),
    "--no-discovery".to_string(),
  ])
  .expect_err("remote listener needs a bearer authentication store");
  assert!(error.contains("requires --pairing-store"));
}
