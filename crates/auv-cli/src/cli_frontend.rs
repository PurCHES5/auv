// Shared frontend for the root `auv` binary.

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use auv_cli_common::{
  TableRow,
  outputs::formats::table::{self, TableOptions},
};

use crate::cli::{CliCommand, DeviceTrustAction, TracingOptions, parse_cli_os, version_text};
use crate::commands::devices::{DeviceProfilesCommand, ProfileWriteArgs};
use crate::commands::pairing::PairingCommand;

pub async fn run_root() -> Result<i32, String> {
  let arguments = env::args_os().skip(1).collect::<Vec<_>>();
  init_diagnostics(verbosity(&arguments));
  let command = parse_cli_os(arguments)?;
  tracing::debug!("parsed CLI command");
  dispatch(command).await
}

fn verbosity(arguments: &[std::ffi::OsString]) -> u8 {
  arguments.iter().fold(0_u8, |count, argument| {
    let Some(argument) = argument.to_str() else {
      return count;
    };
    if argument == "--verbose" {
      count.saturating_add(1)
    } else if argument.starts_with('-') && !argument.starts_with("--") && argument[1..].bytes().all(|byte| byte == b'v') {
      count.saturating_add(u8::try_from(argument.len() - 1).unwrap_or(u8::MAX))
    } else {
      count
    }
  })
}

fn init_diagnostics(verbosity: u8) {
  let level = match verbosity {
    0 => return,
    1 => "info",
    2 => "debug",
    _ => "trace",
  };
  // Keep dependency-level HTTP/gRPC tracing disabled: it may include request
  // metadata or bodies containing bootstrap and bearer credentials.
  let filter = tracing_subscriber::EnvFilter::new(format!("off,auv_cli={level}"));
  let _ = tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr).with_target(false).try_init();
}

pub fn exit_status(result: Result<i32, String>) -> i32 {
  match result {
    Ok(exit_code) => exit_code,
    Err(error) => {
      eprintln!("error: {error}");
      1
    }
  }
}

pub(crate) async fn dispatch(command: CliCommand) -> Result<i32, String> {
  if matches!(&command, CliCommand::Version) {
    print!("{}", version_text());
    return Ok(0);
  }

  let project_root = env::current_dir().map_err(|error| format!("failed to resolve current directory: {error}"))?;
  if let CliCommand::XtaskGenerateSwiftBridge = &command {
    let outputs = crate::xtask::generate_swift_bridge_for_ide(&project_root)?;
    println!("generated Swift bridge files for IDE indexing");
    for output in outputs {
      println!("output: {output}");
    }
    return Ok(0);
  }

  if let CliCommand::McpServe = &command {
    crate::mcp::serve_stdio(project_root.clone()).await?;
    return Ok(0);
  }

  if let CliCommand::PermissionCheck { json } = &command {
    run_permission_check(*json)?;
    return Ok(0);
  }

  if let CliCommand::Pairing {
    store,
    endpoint,
    command,
  } = &command
  {
    run_pairing(&project_root, store.as_deref(), endpoint.as_deref(), command).await?;
    return Ok(0);
  }

  if let CliCommand::DeviceTrust {
    store,
    device,
    action,
  } = &command
  {
    run_device_trust(&project_root, store.as_deref(), device, *action)?;
    return Ok(0);
  }

  if let CliCommand::DeviceProfiles { command } = &command {
    run_device_profiles(command)?;
    return Ok(0);
  }

  if let CliCommand::Serve {
    listeners,
    pairing_store,
    store_root,
    discovery_file,
    no_discovery,
    daemon_idle_timeout,
    runner_providers,
  } = &command
  {
    let listeners = if listeners.is_empty() {
      vec![crate::daemon::default_local_listener(
        discovery_file.as_deref(),
      )?]
    } else {
      listeners.clone()
    };
    let listens =
      listeners.iter().map(|listener| crate::daemon::parse_listener(listener, pairing_store.is_some())).collect::<Result<Vec<_>, _>>()?;
    return crate::daemon::serve_foreground(
      listens,
      resolve_store_root(&project_root, store_root.as_ref()),
      pairing_store.as_ref().map(|path| resolve_path(&project_root, path)),
      discovery_file.clone(),
      *no_discovery,
      *daemon_idle_timeout,
      runner_providers,
    )
    .await;
  }

  if let CliCommand::ApiServerServe {
    host,
    port,
    remote_listen,
    pairing_store,
    #[cfg(unix)]
    unix_socket,
    store_root,
    discovery_file,
    no_discovery,
    daemon_idle_timeout,
    runner_providers,
  } = &command
  {
    let store_root = resolve_store_root(&project_root, store_root.as_ref());
    let listen = if let Some(remote_listen) = remote_listen {
      auv_api_server::server::ListenEndpoint::Remote {
        host: remote_listen.clone(),
        port: *port,
      }
    } else {
      #[cfg(unix)]
      if let Some(path) = unix_socket {
        auv_api_server::server::ListenEndpoint::Unix {
          path: resolve_path(&project_root, path),
        }
      } else {
        auv_api_server::server::ListenEndpoint::Tcp {
          host: host.clone(),
          port: *port,
        }
      }
      #[cfg(not(unix))]
      auv_api_server::server::ListenEndpoint::Tcp {
        host: host.clone(),
        port: *port,
      }
    };
    return crate::daemon::serve_foreground(
      vec![listen],
      store_root,
      pairing_store.as_ref().map(|path| resolve_path(&project_root, path)),
      discovery_file.clone(),
      *no_discovery,
      *daemon_idle_timeout,
      runner_providers,
    )
    .await;
  }

  if let CliCommand::DeviceList {
    endpoint,
    json,
    parent_context,
  } = &command
  {
    return run_device_list(endpoint.as_deref(), parent_context.device_id.as_deref(), parent_context.device_name.as_deref(), *json).await;
  }
  if let CliCommand::DeviceGet {
    endpoint,
    device_id,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    if context.device_id.as_deref().is_some_and(|selected| !id_matches(selected, device_id)) {
      return Err(format!("Device argument {device_id:?} conflicts with root Device selection"));
    }
    return run_device_get(endpoint.as_deref(), device_id, *json).await;
  }
  if let CliCommand::RunnerCreate {
    endpoint,
    runner_class,
    lifecycle,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_runner_create(endpoint.as_deref(), context.device_id.as_deref(), runner_class, *lifecycle, *json).await;
  }
  if let CliCommand::RunnerList {
    endpoint,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_runner_list(endpoint.as_deref(), context.device_id.as_deref(), *json).await;
  }
  if let CliCommand::RunnerClassList {
    endpoint,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_runner_class_list(endpoint.as_deref(), context.device_id.as_deref(), *json).await;
  }
  if let CliCommand::RunnerGet {
    endpoint,
    runner_id,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_runner_get(endpoint.as_deref(), context.device_id.as_deref(), runner_id, *json).await;
  }
  if let CliCommand::RunnerStop {
    endpoint,
    runner_id,
    timeout,
    force,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_runner_stop(endpoint.as_deref(), context.device_id.as_deref(), runner_id, *timeout, *force, *json).await;
  }
  if let CliCommand::RunCreate {
    endpoint,
    device_ids,
    json,
    parent_context,
  } = &command
  {
    if parent_context.run_id.is_some() {
      return Err("root --run cannot be combined with `auv run create`".to_string());
    }
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    let inherited_device_ids = if device_ids.is_empty() {
      context.device_id.as_ref().map_or_else(Vec::new, |device_id| vec![device_id.clone()])
    } else {
      device_ids.clone()
    };
    return run_create(endpoint.as_deref(), &inherited_device_ids, *json).await;
  }
  if let CliCommand::RunList {
    endpoint,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    return run_list(endpoint.as_deref(), context.device_id.as_deref(), context.run_id.as_deref(), *json).await;
  }
  if let CliCommand::RunGet {
    endpoint,
    run_id,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    validate_run_argument(run_id, &context)?;
    return run_get(endpoint.as_deref(), context.device_id.as_deref(), run_id, *json).await;
  }
  if let CliCommand::RunStop {
    endpoint,
    run_id,
    outcome,
    json,
    parent_context,
  } = &command
  {
    let context = crate::plugin::resolve_builtin_context(parent_context, endpoint.as_deref()).await?;
    validate_run_argument(run_id, &context)?;
    return run_stop(endpoint.as_deref(), context.device_id.as_deref(), run_id, *outcome, *json).await;
  }

  let mut exit_code = 0;
  match command {
    CliCommand::Help(help) => {
      print!("{help}");
    }
    CliCommand::Version => unreachable!("version is handled before runtime setup"),
    CliCommand::PermissionCheck { .. } => {
      unreachable!("permission check is handled before runtime setup")
    }
    CliCommand::XtaskGenerateSwiftBridge => unreachable!("xtask is handled before runtime setup"),
    CliCommand::InvokeHelp { command_id } => {
      let registry = auv_cli_invoke::default_registry();
      if let Some(command_id) = command_id {
        let command = registry
          .resolve(&command_id)
          .ok_or_else(|| format!("unknown command {command_id}; use `auv invoke --help` to inspect available entries"))?;
        print!("{}", auv_cli_invoke::render_command_help(command));
      } else {
        print!("{}", auv_cli_invoke::render_help_index(&registry));
      }
    }
    CliCommand::Invoke {
      request,
      typed_args,
      tracing,
      output,
      parent_context,
    } => {
      let authority = build_cli_tracing(&project_root, &tracing)?;
      let registry = auv_cli_invoke::default_registry();
      let command =
        registry.resolve(&request.command_id).cloned().ok_or_else(|| format!("unknown invoke command: {}", request.command_id))?;
      // TODO(selected-invoke-dry-run): validate Device/Run selection without
      // creating a Run once the control plane has a side-effect-free resolve
      // operation. The current dry-run remains local to preserve its no-I/O
      // contract.
      let selected_context = if parent_context != crate::cli::ParentContextOptions::default() && !request.dry_run {
        Some(crate::plugin::resolve_invoke_context(&parent_context).await?)
      } else {
        None
      };
      let remote_context = selected_context.as_ref().map(|resolved| resolved.context.clone());
      let input = auv_cli_invoke::InvokeCommandInput {
        command_id: request.command_id.clone(),
        target_application_id: request.target.application_id,
        inputs: request.inputs,
        typed_args: Some(typed_args),
        dry_run: request.dry_run,
        cancellation: auv_cli_invoke::InvokeCancellation::new(),
      };
      let invoked_command = command.clone();
      let run_id = tracing_run_id_for_selected_context(selected_context.as_ref())?;
      let root = auv_tracing::dispatcher::with_default(&authority.dispatch, || auv_tracing::Context::root(run_id));
      let future = root.in_scope(|| async move {
        auv_tracing::emit_event!(InvokeFrontendLifecycle { frontend: "cli" });
        match remote_context {
          Some(context) => auv_cli_invoke::runner::invoke(input, context).await,
          None => invoked_command.invoke(input).await,
        }
      });
      let mut direct_result = root.instrument(future).await;
      if let Some(context) = selected_context
        && let Err(error) = context.finish(direct_result.is_ok()).await
      {
        if direct_result.is_ok() {
          direct_result = Err(error);
        } else {
          eprintln!("warning: failed to finalize the selected invoke Run: {error}");
        }
      }
      if let Some(failure) = authority.dispatch.flush().await.err().map(|error| error.to_string()) {
        eprintln!("warning: invoke recording failure for run {run_id}: {failure}");
      }
      let artifact_paths = direct_result
        .as_ref()
        .ok()
        .into_iter()
        .flat_map(auv_cli_invoke::InvokeCommandOutput::artifacts)
        .map(|metadata| (metadata.uri().clone(), authority.store.artifact_path(metadata)))
        .collect::<Vec<_>>();
      let result = auv_cli_invoke::InvokeResult::from_command_result(run_id, &command, direct_result).with_artifact_paths(artifact_paths);
      let outcome = auv_cli_invoke::render_invoke_result(&result, output)?;
      exit_code = outcome.exit_code;
    }
    CliCommand::McpServe => {
      unreachable!("mcp serve is handled before runtime setup")
    }
    CliCommand::ApiServerServe { .. } => {
      unreachable!("api-server serve is handled before runtime setup")
    }
    CliCommand::Serve { .. } => unreachable!("serve is handled before runtime setup"),
    CliCommand::DeviceList { .. } | CliCommand::DeviceGet { .. } | CliCommand::DeviceProfiles { .. } | CliCommand::DeviceTrust { .. } => {
      unreachable!("Device commands are handled before runtime setup")
    }
    CliCommand::RunnerCreate { .. }
    | CliCommand::RunnerList { .. }
    | CliCommand::RunnerClassList { .. }
    | CliCommand::RunnerGet { .. }
    | CliCommand::RunnerStop { .. } => unreachable!("Runner commands are handled before runtime setup"),
    CliCommand::RunCreate { .. } | CliCommand::RunList { .. } | CliCommand::RunGet { .. } | CliCommand::RunStop { .. } => {
      unreachable!("Run commands are handled before runtime setup")
    }
    CliCommand::Pairing { .. } => {
      unreachable!("pairing is handled before runtime setup")
    }
    CliCommand::PluginList => {
      exit_code = crate::plugin::list()?;
    }
    CliCommand::External {
      command_name,
      arguments,
      parent_context,
    } => {
      exit_code = crate::plugin::execute(&command_name, &arguments, &parent_context).await?;
    }
  }

  Ok(exit_code)
}

async fn connected_api_client(explicit: Option<&str>) -> Result<Option<auv_api_client::protocol::grpc::Client>, String> {
  let Some(endpoint) = auv::discovery::resolve(explicit).map_err(|error| error.to_string())? else {
    return Ok(None);
  };
  let endpoint_display = endpoint.to_string();
  tracing::info!(endpoint = %endpoint_display, "connecting to AUV API endpoint");
  auv_api_client::protocol::grpc::Client::connect(endpoint)
    .await
    .map(Some)
    .map_err(|error| format!("failed to connect to AUV API server at {endpoint_display}: {error}"))
}

async fn resolve_device_id(client: &auv_api_client::protocol::grpc::Client, selector: &str) -> Result<String, String> {
  let devices = client.devices().list_devices().await.map_err(|status| format!("ListDevices failed while resolving ID: {status}"))?;
  resolve_id("Device", selector, devices.iter().filter_map(|device| device.r#ref.as_ref().map(|reference| reference.device_id.as_str())))
}

#[derive(TableRow)]
struct DeviceTableRow {
  #[table(header = "DEVICE ID")]
  device_id: String,
  name: Option<String>,
  platform: Option<String>,
  local: bool,
  status: String,
  profile: Option<String>,
}

#[derive(TableRow)]
struct DeviceProfileTableRow {
  #[table(header = "PROFILE")]
  config_profile: String,
  #[table(header = "DEVICE ID")]
  device_id: String,
  name: String,
  endpoint: String,
}

#[derive(TableRow)]
struct RunnerClassTableRow {
  #[table(header = "CLASS")]
  runner_class: String,
  name: String,
  available: bool,
  #[table(header = "DEVICE ID")]
  device_id: Option<String>,
  lifecycles: String,
}

#[derive(TableRow)]
struct RunTableRow {
  #[table(header = "RUN ID")]
  run_id: String,
  phase: String,
  #[table(header = "DEVICE IDS")]
  device_ids: String,
}

#[derive(TableRow)]
struct RunnerTableRow {
  #[table(header = "RUNNER ID")]
  runner_id: String,
  class: String,
  phase: String,
  pid: Option<u32>,
  #[table(header = "DEVICE ID")]
  device_id: Option<String>,
  lifecycle: String,
  #[table(header = "OPERATIONS")]
  active_operations: u64,
}

fn print_table<R: table::TableRow>(rows: &[R], empty_message: &'static str) {
  println!("{}", table::render(rows, TableOptions::default().empty_message(empty_message)));
}

fn short_enum_name(value: &str, prefix: &str) -> String {
  value.strip_prefix(prefix).unwrap_or(value).to_ascii_lowercase().replace('_', "-")
}

fn compact_id(value: &str) -> String {
  value.split_once('_').map_or(value, |(_, suffix)| suffix).chars().filter(|character| *character != '-').collect()
}

fn short_id(value: &str) -> String {
  compact_id(value).chars().take(12).collect()
}

fn random_identity() -> Result<String, String> {
  let mut bytes = [0_u8; 32];
  getrandom::fill(&mut bytes).map_err(|error| format!("failed to generate identity: {error}"))?;
  Ok(hex::encode(bytes))
}

fn id_matches(value: &str, selector: &str) -> bool {
  value == selector || compact_id(value).starts_with(&selector.replace('-', ""))
}

fn resolve_id<'a>(kind: &str, selector: &str, values: impl Iterator<Item = &'a str>) -> Result<String, String> {
  let matches = values.filter(|value| id_matches(value, selector)).collect::<Vec<_>>();
  match matches.as_slice() {
    [] => Err(format!("unknown {kind} ID {selector:?}")),
    [value] => Ok((*value).to_string()),
    _ => Err(format!("ambiguous {kind} ID prefix {selector:?}; provide more characters")),
  }
}

fn device_table_row(device: &auv_api_proto::auv::api::daemon::v1::Device, status: &str) -> DeviceTableRow {
  let platform = auv_api_proto::auv::api::daemon::v1::DevicePlatform::try_from(device.platform)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::DevicePlatform::Unspecified)
    .as_str_name();
  DeviceTableRow {
    device_id: device.r#ref.as_ref().map(|reference| short_id(&reference.device_id)).unwrap_or_else(|| "<missing>".to_string()),
    name: (!device.name.is_empty()).then(|| device.name.clone()),
    platform: Some(short_enum_name(platform, "DEVICE_PLATFORM_")),
    local: device.local,
    status: status.to_string(),
    profile: None,
  }
}

fn configured_device_probe_table_row(probe: &ConfiguredDeviceProbe) -> DeviceTableRow {
  let platform = probe.remote.as_ref().map(|device| {
    let platform = auv_api_proto::auv::api::daemon::v1::DevicePlatform::try_from(device.platform)
      .unwrap_or(auv_api_proto::auv::api::daemon::v1::DevicePlatform::Unspecified)
      .as_str_name();
    short_enum_name(platform, "DEVICE_PLATFORM_")
  });
  let name = probe
    .remote
    .as_ref()
    .and_then(|device| (!device.name.is_empty()).then(|| device.name.clone()))
    .or_else(|| (!probe.profile.device_name().is_empty()).then(|| probe.profile.device_name().to_string()));
  DeviceTableRow {
    device_id: short_id(probe.profile.device_id()),
    name,
    platform,
    local: false,
    status: probe.status.clone(),
    profile: Some(probe.profile.config_profile().to_string()),
  }
}

fn device_profile_table_row(device: &auv::profile::ConfiguredDevice) -> DeviceProfileTableRow {
  DeviceProfileTableRow {
    config_profile: device.config_profile().to_string(),
    device_id: short_id(device.device_id()),
    name: device.device_name().to_string(),
    endpoint: device.endpoint().to_string(),
  }
}

async fn run_device_list(endpoint: Option<&str>, device_id: Option<&str>, device_name: Option<&str>, json: bool) -> Result<i32, String> {
  let mut devices = match connected_api_client(endpoint).await {
    Ok(Some(client)) => client.devices().list_devices().await.map_err(|status| format!("ListDevices failed: {status}"))?,
    Ok(None) => Vec::new(),
    Err(error) if endpoint.is_none() => {
      eprintln!("warning: local AUV daemon is unavailable: {error}");
      Vec::new()
    }
    Err(error) => return Err(error),
  };
  if let Some(device_id) = device_id {
    devices.retain(|device| device.r#ref.as_ref().is_some_and(|reference| id_matches(&reference.device_id, device_id)));
  }
  if let Some(device_name) = device_name {
    devices.retain(|device| device.name == device_name);
  }
  let profile_store = auv::profile::ProfileStore::from_env().map_err(|error| error.to_string())?;
  let profiles = match profile_store.list_devices() {
    Ok(profiles) => profiles,
    Err(auv::profile::ProfileError::Open { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => Vec::new(),
    Err(error) => return Err(error.to_string()),
  };
  let profiles = profiles
    .into_iter()
    .filter(|profile| {
      device_id.is_none_or(|id| id_matches(profile.device_id(), id)) && device_name.is_none_or(|name| profile.device_name() == name)
    })
    .collect();
  let profiles = probe_configured_devices(profile_store, profiles).await;
  if json {
    let mut values = devices
      .iter()
      .map(|device| {
        let mut value = device_json(device);
        value["source"] = serde_json::json!("daemon");
        value["status"] = serde_json::json!("online");
        value
      })
      .collect::<Vec<_>>();
    for probe in &profiles {
      if devices.iter().any(|device| device.r#ref.as_ref().is_some_and(|reference| reference.device_id == probe.profile.device_id())) {
        continue;
      }
      values.push(configured_device_probe_json(probe));
    }
    println!("{}", serde_json::to_string_pretty(&values).map_err(|error| format!("failed to encode Device list: {error}"))?);
  } else {
    let mut rows = devices.iter().map(|device| device_table_row(device, "online")).collect::<Vec<_>>();
    for probe in &profiles {
      if devices.iter().any(|device| device.r#ref.as_ref().is_some_and(|reference| reference.device_id == probe.profile.device_id())) {
        continue;
      }
      rows.push(configured_device_probe_table_row(probe));
    }
    print_table(&rows, "(no devices)");
  }
  Ok(0)
}

async fn probe_configured_devices(
  store: auv::profile::ProfileStore,
  profiles: Vec<auv::profile::ConfiguredDevice>,
) -> Vec<ConfiguredDeviceProbe> {
  let mut probes = tokio::task::JoinSet::new();
  for (index, profile) in profiles.into_iter().enumerate() {
    let store = store.clone();
    probes.spawn(async move {
      let context = auv::AuvContext {
        config_profile: Some(profile.config_profile().to_string()),
        ..Default::default()
      };
      let (status, remote) = match auv::Client::from_context_with_profiles(context, &store).await {
        Ok(client) => {
          let remote =
            client.grpc().devices().get_device(profile.device_id().to_string()).await.map_err(auv::ContextError::RemoteDeviceList);
          match remote {
            Ok(remote) => ("online", Some(remote)),
            Err(error) => (configured_device_error_status(&error), None),
          }
        }
        Err(error) => {
          let status = configured_device_error_status(&error);
          tracing::debug!(
            profile = profile.config_profile(),
            endpoint = %profile.endpoint(),
            status,
            error = %error,
            "configured Device probe failed"
          );
          (status, None)
        }
      };
      (
        index,
        ConfiguredDeviceProbe {
          profile,
          status: status.to_string(),
          remote,
        },
      )
    });
  }

  let mut results = Vec::new();
  while let Some(result) = probes.join_next().await {
    match result {
      Ok(result) => results.push(result),
      Err(error) => tracing::debug!(error = %error, "configured Device probe task failed"),
    }
  }
  results.sort_by_key(|(index, _)| *index);
  results.into_iter().map(|(_, probe)| probe).collect()
}

struct ConfiguredDeviceProbe {
  profile: auv::profile::ConfiguredDevice,
  status: String,
  remote: Option<auv_api_proto::auv::api::daemon::v1::Device>,
}

fn configured_device_error_status(error: &auv::ContextError) -> &'static str {
  match error {
    auv::ContextError::Connect(_) | auv::ContextError::PairedConnect(_) => "offline",
    auv::ContextError::RemoteDeviceList(status) if matches!(status.code(), tonic::Code::Unauthenticated | tonic::Code::PermissionDenied) => {
      "unauthorized"
    }
    auv::ContextError::RemoteDeviceList(status)
      if matches!(status.code(), tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::Cancelled) =>
    {
      "offline"
    }
    auv::ContextError::Profile(_) | auv::ContextError::ProfileEndpointMismatch { .. } | auv::ContextError::CanonicalDeviceMissing(_) => {
      "invalid"
    }
    _ => "error",
  }
}

fn configured_device_json(device: &auv::profile::ConfiguredDevice, status: &str) -> serde_json::Value {
  configured_device_probe_json(&ConfiguredDeviceProbe {
    profile: device.clone(),
    status: status.to_string(),
    remote: None,
  })
}

fn configured_device_probe_json(probe: &ConfiguredDeviceProbe) -> serde_json::Value {
  let platform = probe.remote.as_ref().map(|device| {
    auv_api_proto::auv::api::daemon::v1::DevicePlatform::try_from(device.platform)
      .unwrap_or(auv_api_proto::auv::api::daemon::v1::DevicePlatform::Unspecified)
      .as_str_name()
  });
  let name = probe
    .remote
    .as_ref()
    .and_then(|device| (!device.name.is_empty()).then_some(device.name.as_str()))
    .unwrap_or_else(|| probe.profile.device_name());
  serde_json::json!({
    "device_id": probe.profile.device_id(),
    "name": name,
    "platform": platform,
    "local": false,
    "source": "configured_profile",
    "status": probe.status,
    "config_profile": probe.profile.config_profile(),
    "endpoint": probe.profile.endpoint().to_string(),
    "labels": probe.remote.as_ref().map(|device| &device.labels),
  })
}

fn run_device_profiles(command: &DeviceProfilesCommand) -> Result<(), String> {
  let store = auv::profile::ProfileStore::from_env().map_err(|error| error.to_string())?;
  match command {
    DeviceProfilesCommand::List(args) => {
      let profiles = match store.list_devices() {
        Ok(profiles) => profiles,
        Err(auv::profile::ProfileError::Open { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.to_string()),
      };
      if args.json {
        let values = profiles.iter().map(|profile| configured_device_json(profile, "configured")).collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&values).map_err(|error| error.to_string())?);
      } else {
        let rows = profiles.iter().map(device_profile_table_row).collect::<Vec<_>>();
        print_table(&rows, "(no device profiles)");
      }
    }
    DeviceProfilesCommand::Get(args) => {
      let profile = store.get_device(&args.name).map_err(|error| error.to_string())?;
      if args.json {
        println!("{}", serde_json::to_string_pretty(&configured_device_json(&profile, "configured")).map_err(|error| error.to_string())?);
      } else {
        print_table(&[device_profile_table_row(&profile)], "(no device profile)");
      }
    }
    DeviceProfilesCommand::Create(args) => {
      store.create(&args.name, profile_inputs(args)).map_err(|error| error.to_string())?;
    }
    DeviceProfilesCommand::Update(args) => {
      store.update(&args.name, profile_inputs(args)).map_err(|error| error.to_string())?;
    }
    DeviceProfilesCommand::Delete(args) => store.delete(&args.name).map_err(|error| error.to_string())?,
  }
  Ok(())
}

fn profile_inputs(args: &ProfileWriteArgs) -> auv::profile::DeviceProfileInput {
  auv::profile::DeviceProfileInput {
    device_id: args.device_id.clone(),
    device_name: args.device_name.clone(),
    endpoint: args.endpoint.clone(),
    device_credential: args.device_credential.clone(),
  }
}

async fn run_device_get(endpoint: Option<&str>, device_id: &str, json: bool) -> Result<i32, String> {
  let Some(client) = connected_api_client(endpoint).await? else {
    return Err("no AUV daemon was discovered".to_string());
  };
  let device_id = resolve_device_id(&client, device_id).await?;
  let device = client.devices().get_device(device_id).await.map_err(|status| format!("GetDevice failed: {status}"))?;
  if json {
    println!("{}", serde_json::to_string_pretty(&device_json(&device)).map_err(|error| format!("failed to encode Device: {error}"))?);
  } else {
    print_table(&[device_table_row(&device, "online")], "(no device)");
  }
  Ok(0)
}

fn device_json(device: &auv_api_proto::auv::api::daemon::v1::Device) -> serde_json::Value {
  let platform = auv_api_proto::auv::api::daemon::v1::DevicePlatform::try_from(device.platform)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::DevicePlatform::Unspecified)
    .as_str_name();
  serde_json::json!({
    "device_id": device.r#ref.as_ref().map(|reference| reference.device_id.as_str()),
    "name": device.name,
    "platform": platform,
    "local": device.local,
    "labels": device.labels,
  })
}

async fn run_runner_create(
  endpoint: Option<&str>,
  device_id: Option<&str>,
  runner_class: &str,
  lifecycle: i32,
  json: bool,
) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let runner = client
    .runners()
    .create_runner(auv_api_proto::auv::api::daemon::v1::CreateRunnerRequest {
      device: device_id.map(|device_id| auv_api_proto::auv::api::daemon::v1::DeviceRef {
        device_id: device_id.to_string(),
      }),
      runner_class: Some(auv_api_proto::auv::api::daemon::v1::RunnerClassRef {
        runner_class: runner_class.to_string(),
      }),
      labels: Default::default(),
      lifecycle,
      idle_timeout: None,
    })
    .await
    .map_err(|status| format!("CreateRunner failed: {status}"))?;
  print_runner(&runner, json)?;
  Ok(0)
}

async fn run_runner_class_list(endpoint: Option<&str>, device_id: Option<&str>, json: bool) -> Result<i32, String> {
  let Some(client) = connected_api_client(endpoint).await? else {
    if json {
      println!("[]");
    } else {
      print_table(&Vec::<RunnerClassTableRow>::new(), "(no runner classes)");
    }
    return Ok(0);
  };
  let classes = client
    .runner_classes()
    .list_runner_classes(device_id.map(|device_id| auv_api_proto::auv::api::daemon::v1::DeviceRef {
      device_id: device_id.to_string(),
    }))
    .await
    .map_err(|status| format!("ListRunnerClasses failed: {status}"))?;
  let values = classes.iter().map(runner_class_json).collect::<Vec<_>>();
  if json {
    println!("{}", serde_json::to_string_pretty(&values).map_err(|error| error.to_string())?);
  } else {
    let rows = classes.iter().map(runner_class_table_row).collect::<Vec<_>>();
    print_table(&rows, "(no runner classes)");
  }
  Ok(0)
}

fn runner_class_table_row(class: &auv_api_proto::auv::api::daemon::v1::RunnerClass) -> RunnerClassTableRow {
  let lifecycles = class
    .supported_lifecycles
    .iter()
    .map(|lifecycle| {
      let lifecycle = auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::try_from(*lifecycle)
        .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::Unspecified)
        .as_str_name();
      short_enum_name(lifecycle, "RUNNER_LIFECYCLE_")
    })
    .collect::<Vec<_>>()
    .join(",");
  RunnerClassTableRow {
    runner_class: class.r#ref.as_ref().map(|reference| reference.runner_class.clone()).unwrap_or_else(|| "<missing>".to_string()),
    name: class.display_name.clone(),
    available: class.available,
    device_id: class.device.as_ref().map(|reference| short_id(&reference.device_id)),
    lifecycles,
  }
}

fn runner_class_json(class: &auv_api_proto::auv::api::daemon::v1::RunnerClass) -> serde_json::Value {
  serde_json::json!({
    "runner_class": class.r#ref.as_ref().map(|reference| reference.runner_class.as_str()),
    "device_id": class.device.as_ref().map(|reference| reference.device_id.as_str()),
    "display_name": class.display_name,
    "available": class.available,
    "supported_lifecycles": class.supported_lifecycles.iter().map(|lifecycle| {
      auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::try_from(*lifecycle)
        .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::Unspecified)
        .as_str_name()
    }).collect::<Vec<_>>(),
  })
}

async fn run_create(endpoint: Option<&str>, device_ids: &[String], json: bool) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let run = client
    .runs()
    .create_run(auv_api_proto::auv::api::daemon::v1::CreateRunRequest {
      devices: device_ids
        .iter()
        .map(|device_id| auv_api_proto::auv::api::daemon::v1::DeviceRef {
          device_id: device_id.clone(),
        })
        .collect(),
      labels: Default::default(),
    })
    .await
    .map_err(|status| format!("CreateRun failed: {status}"))?;
  print_run(&run, json)?;
  Ok(0)
}

async fn run_list(endpoint: Option<&str>, device_id: Option<&str>, run_id: Option<&str>, json: bool) -> Result<i32, String> {
  let Some(client) = connected_api_client(endpoint).await? else {
    if json {
      println!("[]");
    } else {
      print_table(&Vec::<RunTableRow>::new(), "(no runs)");
    }
    return Ok(0);
  };
  let mut runs = client.runs().list_runs().await.map_err(|status| format!("ListRuns failed: {status}"))?;
  if let Some(device_id) = device_id {
    runs.retain(|run| run.devices.iter().any(|device| device.device_id == device_id));
  }
  if let Some(run_id) = run_id {
    runs.retain(|run| run.r#ref.as_ref().is_some_and(|reference| id_matches(&reference.run_id, run_id)));
  }
  if json {
    println!("{}", serde_json::to_string_pretty(&runs.iter().map(run_json).collect::<Vec<_>>()).map_err(|error| error.to_string())?);
  } else {
    let rows = runs.iter().map(run_table_row).collect::<Vec<_>>();
    print_table(&rows, "(no runs)");
  }
  Ok(0)
}

async fn run_get(endpoint: Option<&str>, device_id: Option<&str>, run_id: &str, json: bool) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let runs = client.runs().list_runs().await.map_err(|status| format!("ListRuns failed while resolving ID: {status}"))?;
  let run_id = resolve_id("Run", run_id, runs.iter().filter_map(|run| run.r#ref.as_ref().map(|reference| reference.run_id.as_str())))?;
  let run = client.runs().get_run(run_id).await.map_err(|status| format!("GetRun failed: {status}"))?;
  validate_run_device(&run, device_id)?;
  print_run(&run, json)?;
  Ok(0)
}

async fn run_stop(endpoint: Option<&str>, device_id: Option<&str>, run_id: &str, outcome: i32, json: bool) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let runs = client.runs().list_runs().await.map_err(|status| format!("ListRuns failed while resolving ID: {status}"))?;
  let run_id = resolve_id("Run", run_id, runs.iter().filter_map(|run| run.r#ref.as_ref().map(|reference| reference.run_id.as_str())))?;
  if device_id.is_some() {
    let run = client.runs().get_run(&run_id).await.map_err(|status| format!("GetRun failed: {status}"))?;
    validate_run_device(&run, device_id)?;
  }
  let outcome = auv_api_proto::auv::api::daemon::v1::RunOutcome::try_from(outcome).map_err(|_| "Run outcome is invalid".to_string())?;
  let run = client.runs().stop_run(run_id, outcome).await.map_err(|status| format!("StopRun failed: {status}"))?;
  print_run(&run, json)?;
  Ok(0)
}

fn validate_run_argument(run_id: &str, context: &auv::AuvContext) -> Result<(), String> {
  if context.run_id.as_deref().is_some_and(|selected| !id_matches(selected, run_id)) {
    return Err(format!("Run argument {run_id:?} conflicts with root --run"));
  }
  Ok(())
}

fn validate_run_device(run: &auv_api_proto::auv::api::daemon::v1::Run, expected_device_id: Option<&str>) -> Result<(), String> {
  if let Some(expected_device_id) = expected_device_id
    && !run.devices.iter().any(|device| device.device_id == expected_device_id)
  {
    return Err(format!("Run is not attached to selected Device {expected_device_id:?}"));
  }
  Ok(())
}

fn print_run(run: &auv_api_proto::auv::api::daemon::v1::Run, json: bool) -> Result<(), String> {
  if json {
    println!("{}", serde_json::to_string_pretty(&run_json(run)).map_err(|error| error.to_string())?);
  } else {
    print_table(&[run_table_row(run)], "(no run)");
  }
  Ok(())
}

fn run_table_row(run: &auv_api_proto::auv::api::daemon::v1::Run) -> RunTableRow {
  let phase = auv_api_proto::auv::api::daemon::v1::RunPhase::try_from(run.phase)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunPhase::Unspecified)
    .as_str_name();
  RunTableRow {
    run_id: run.r#ref.as_ref().map(|reference| short_id(&reference.run_id)).unwrap_or_else(|| "<missing>".to_string()),
    phase: short_enum_name(phase, "RUN_PHASE_"),
    device_ids: run.devices.iter().map(|device| short_id(&device.device_id)).collect::<Vec<_>>().join(","),
  }
}

fn run_json(run: &auv_api_proto::auv::api::daemon::v1::Run) -> serde_json::Value {
  let phase = auv_api_proto::auv::api::daemon::v1::RunPhase::try_from(run.phase)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunPhase::Unspecified)
    .as_str_name();
  serde_json::json!({
    "run_id": run.r#ref.as_ref().map(|reference| reference.run_id.as_str()),
    "phase": phase,
    "device_ids": run.devices.iter().map(|device| device.device_id.as_str()).collect::<Vec<_>>(),
    "labels": run.labels,
    "created_at": timestamp_json(run.created_at.as_ref()),
  })
}

async fn run_runner_list(endpoint: Option<&str>, device_id: Option<&str>, json: bool) -> Result<i32, String> {
  let Some(client) = connected_api_client(endpoint).await? else {
    if json {
      println!("[]");
    } else {
      print_table(&Vec::<RunnerTableRow>::new(), "(no runners)");
    }
    return Ok(0);
  };
  let mut runners = client.runners().list_runners().await.map_err(|status| format!("ListRunners failed: {status}"))?;
  if let Some(device_id) = device_id {
    runners.retain(|runner| runner.device.as_ref().is_some_and(|device| device.device_id == device_id));
  }
  if json {
    println!("{}", serde_json::to_string_pretty(&runners.iter().map(runner_json).collect::<Vec<_>>()).map_err(|error| error.to_string())?);
  } else {
    let rows = runners.iter().map(runner_table_row).collect::<Vec<_>>();
    print_table(&rows, "(no runners)");
  }
  Ok(0)
}

async fn run_runner_get(endpoint: Option<&str>, device_id: Option<&str>, runner_id: &str, json: bool) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let runners = client.runners().list_runners().await.map_err(|status| format!("ListRunners failed while resolving ID: {status}"))?;
  let runner_id = resolve_id(
    "Runner",
    runner_id,
    runners.iter().filter_map(|runner| runner.r#ref.as_ref().map(|reference| reference.runner_id.as_str())),
  )?;
  let runner = client.runners().get_runner(runner_id).await.map_err(|status| format!("GetRunner failed: {status}"))?;
  validate_runner_device(&runner, device_id)?;
  print_runner(&runner, json)?;
  Ok(0)
}

async fn run_runner_stop(
  endpoint: Option<&str>,
  device_id: Option<&str>,
  runner_id: &str,
  timeout: Option<u64>,
  force: bool,
  json: bool,
) -> Result<i32, String> {
  let client = required_client(endpoint).await?;
  let runners = client.runners().list_runners().await.map_err(|status| format!("ListRunners failed while resolving ID: {status}"))?;
  let runner_id = resolve_id(
    "Runner",
    runner_id,
    runners.iter().filter_map(|runner| runner.r#ref.as_ref().map(|reference| reference.runner_id.as_str())),
  )?;
  if device_id.is_some() {
    let runner = client.runners().get_runner(&runner_id).await.map_err(|status| format!("GetRunner failed: {status}"))?;
    validate_runner_device(&runner, device_id)?;
  }
  let grace_period = timeout
    .map(|seconds| {
      i64::try_from(seconds)
        .map(|seconds| prost_types::Duration { seconds, nanos: 0 })
        .map_err(|_| "--timeout exceeds the protobuf duration range".to_string())
    })
    .transpose()?;
  let runner = client
    .runners()
    .delete_runner_with_options(runner_id, grace_period, force)
    .await
    .map_err(|status| format!("DeleteRunner failed: {status}"))?;
  print_runner(&runner, json)?;
  Ok(0)
}

fn validate_runner_device(runner: &auv_api_proto::auv::api::daemon::v1::Runner, expected_device_id: Option<&str>) -> Result<(), String> {
  if let Some(expected_device_id) = expected_device_id
    && runner.device.as_ref().is_none_or(|device| device.device_id != expected_device_id)
  {
    return Err(format!("Runner is not owned by selected Device {expected_device_id:?}"));
  }
  Ok(())
}

async fn required_client(endpoint: Option<&str>) -> Result<auv_api_client::protocol::grpc::Client, String> {
  connected_api_client(endpoint).await?.ok_or_else(|| "no AUV daemon was discovered".to_string())
}

fn print_runner(runner: &auv_api_proto::auv::api::daemon::v1::Runner, json: bool) -> Result<(), String> {
  if json {
    println!("{}", serde_json::to_string_pretty(&runner_json(runner)).map_err(|error| error.to_string())?);
  } else {
    print_table(&[runner_table_row(runner)], "(no runner)");
  }
  Ok(())
}

fn runner_table_row(runner: &auv_api_proto::auv::api::daemon::v1::Runner) -> RunnerTableRow {
  let phase = auv_api_proto::auv::api::daemon::v1::RunnerPhase::try_from(runner.phase)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunnerPhase::Unspecified)
    .as_str_name();
  let lifecycle = auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::try_from(runner.lifecycle)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::Unspecified)
    .as_str_name();
  RunnerTableRow {
    runner_id: runner.r#ref.as_ref().map(|reference| short_id(&reference.runner_id)).unwrap_or_else(|| "<missing>".to_string()),
    class: runner.runner_class.as_ref().map(|reference| reference.runner_class.clone()).unwrap_or_else(|| "<missing>".to_string()),
    phase: short_enum_name(phase, "RUNNER_PHASE_"),
    pid: (runner.process_id != 0).then_some(runner.process_id),
    device_id: runner.device.as_ref().map(|reference| short_id(&reference.device_id)),
    lifecycle: short_enum_name(lifecycle, "RUNNER_LIFECYCLE_"),
    active_operations: runner.active_operations,
  }
}

fn runner_json(runner: &auv_api_proto::auv::api::daemon::v1::Runner) -> serde_json::Value {
  let phase = auv_api_proto::auv::api::daemon::v1::RunnerPhase::try_from(runner.phase)
    .unwrap_or(auv_api_proto::auv::api::daemon::v1::RunnerPhase::Unspecified)
    .as_str_name();
  serde_json::json!({
    "runner_id": runner.r#ref.as_ref().map(|reference| reference.runner_id.as_str()),
    "device_id": runner.device.as_ref().map(|reference| reference.device_id.as_str()),
    "runner_class": runner.runner_class.as_ref().map(|reference| reference.runner_class.as_str()),
    "phase": phase,
    "process_id": runner.process_id,
    "labels": runner.labels,
  })
}

fn timestamp_json(timestamp: Option<&prost_types::Timestamp>) -> serde_json::Value {
  timestamp.map_or(serde_json::Value::Null, |timestamp| serde_json::json!({ "seconds": timestamp.seconds, "nanos": timestamp.nanos }))
}

#[derive(serde::Serialize)]
struct PermissionCheckReport {
  platform: &'static str,
  process_id: u32,
  executable: Option<String>,
  accessibility: &'static str,
  screen_recording_preflight: &'static str,
  screen_capture_kit: &'static str,
  all_ok: bool,
  warnings: Vec<String>,
  recommendation: String,
}

fn run_permission_check(json: bool) -> Result<(), String> {
  let report = collect_permission_check()?;

  if json {
    println!("{}", serde_json::to_string_pretty(&report).map_err(|error| format!("failed to encode permission report: {error}"))?);
  } else {
    print_permission_check_report(&report);
  }

  Ok(())
}

#[cfg(target_os = "macos")]
fn collect_permission_check() -> Result<PermissionCheckReport, String> {
  let native = auv_driver_macos::native::permission::probe_native_permissions()?;
  let all_ok = native.accessibility == "granted" && native.screen_capture_kit == "granted";
  let mut warnings = Vec::new();

  if native.screen_recording == "missing" && native.screen_capture_kit == "granted" {
    warnings.push(
      "CGPreflightScreenCaptureAccess reports missing, but the ScreenCaptureKit probe works; this can happen when the launch host owns TCC attribution."
        .to_string(),
    );
  }

  Ok(PermissionCheckReport {
    platform: "macos",
    process_id: std::process::id(),
    executable: env::current_exe().ok().map(|path| path.display().to_string()),
    accessibility: native.accessibility,
    screen_recording_preflight: native.screen_recording,
    screen_capture_kit: native.screen_capture_kit,
    all_ok,
    warnings,
    recommendation: permission_recommendation(native.accessibility, native.screen_capture_kit),
  })
}

#[cfg(not(target_os = "macos"))]
fn collect_permission_check() -> Result<PermissionCheckReport, String> {
  Err("permission check is currently implemented only for macOS".to_string())
}

fn permission_recommendation(accessibility: &str, screen_capture_kit: &str) -> String {
  match (accessibility, screen_capture_kit) {
    ("granted", "granted") => "AUV has the macOS permissions needed for capture and AX-backed automation.".to_string(),
    ("missing", "missing") => {
      "Grant Accessibility and Screen Recording to the terminal or app that launches auv, then rerun this check.".to_string()
    }
    ("missing", _) => "Grant Accessibility to the terminal or app that launches auv, then rerun this check.".to_string(),
    (_, "missing") => "Grant Screen Recording to the terminal or app that launches auv, then rerun this check.".to_string(),
    _ => "Review the permission statuses above before running desktop automation.".to_string(),
  }
}

fn print_permission_check_report(report: &PermissionCheckReport) {
  println!("AUV permission check");
  println!("platform: {}", report.platform);
  println!("process: {}", report.process_id);
  if let Some(executable) = &report.executable {
    println!("executable: {executable}");
  }
  println!("accessibility: {}", permission_status_line(report.accessibility));
  println!("screen recording preflight: {}", permission_status_line(report.screen_recording_preflight));
  println!("screen capture kit probe: {}", permission_status_line(report.screen_capture_kit));
  for warning in &report.warnings {
    println!("warning: {warning}");
  }
  println!("all ok: {}", report.all_ok);
  println!("recommendation: {}", report.recommendation);
}

fn permission_status_line(status: &str) -> String {
  match status {
    "granted" => "[ok] granted".to_string(),
    "missing" => "[missing] missing".to_string(),
    other => format!("[unknown] {other}"),
  }
}

pub(crate) fn resolve_store_root(project_root: &Path, explicit: Option<&PathBuf>) -> PathBuf {
  explicit.cloned().unwrap_or_else(|| project_root.join(".auv").join("store"))
}

fn resolve_path(project_root: &Path, path: &Path) -> PathBuf {
  if path.is_absolute() {
    path.to_path_buf()
  } else {
    project_root.join(path)
  }
}

fn run_device_trust(project_root: &Path, store: Option<&Path>, selector: &str, action: DeviceTrustAction) -> Result<(), String> {
  use auv_api_server::auth::PairingStore;

  let selector = selector.trim();
  if selector.is_empty() {
    return Err("Device selector must not be empty".to_string());
  }
  let store_path = store.map_or_else(|| project_root.join(".auv").join("pairings.json"), |path| resolve_path(project_root, path));
  // TODO(live-pairing-admin): Device trust mutation currently owns the store
  // lock and therefore runs only while the daemon is stopped. Re-open this
  // boundary when an audited, owner-authorized local administration RPC lands.
  let store =
    PairingStore::open(store_path.clone()).map_err(|error| format!("failed to open pairing store {}: {error}", store_path.display()))?;
  let records = store.list();
  let pair_id = if records.iter().any(|record| record.pair_id == selector) {
    selector.to_string()
  } else {
    let mut matches = records.iter().filter(|record| record.label == selector).map(|record| record.pair_id.clone()).collect::<Vec<_>>();
    matches.sort();
    match matches.as_slice() {
      [] => return Err(format!("unknown paired Device {selector:?}")),
      [pair_id] => pair_id.clone(),
      _ => {
        return Err(format!("paired Device name {selector:?} is ambiguous; use one of these stable IDs: {}", matches.join(", ")));
      }
    }
  };

  match action {
    DeviceTrustAction::Unpair => store.remove_pair(&pair_id).map_err(|error| format!("failed to unpair Device {pair_id}: {error}"))?,
    DeviceTrustAction::Enable => store.set_enabled(&pair_id, true).map_err(|error| format!("failed to enable Device {pair_id}: {error}"))?,
    DeviceTrustAction::Disable => {
      store.set_enabled(&pair_id, false).map_err(|error| format!("failed to disable Device {pair_id}: {error}"))?
    }
  }
  Ok(())
}

async fn run_pairing(project_root: &Path, store: Option<&Path>, endpoint: Option<&str>, command: &PairingCommand) -> Result<(), String> {
  use auv_api_server::auth::PairingStore;

  let store_path = store.map_or_else(|| project_root.join(".auv").join("pairings.json"), |path| resolve_path(project_root, path));
  // TODO(live-pairing-resource-admin): enable operations below still
  // require a stopped daemon. Move them to owner-checked Device administration
  // RPCs when that resource contract is approved. Token creation and enrollment
  // already use the live daemon protocol.
  match command {
    PairingCommand::CreateToken { ttl } => {
      let client = connected_api_client(endpoint).await?.ok_or_else(|| "no local AUV daemon was discovered".to_string())?;
      let response = client
        .pairing()
        .create_pairing_token(auv_api_proto::auv::api::daemon::v1::CreatePairingTokenRequest {
          ttl: ttl.map(|seconds| prost_types::Duration {
            seconds: i64::try_from(seconds).unwrap_or(i64::MAX),
            nanos: 0,
          }),
        })
        .await
        .map_err(|status| format!("CreatePairingToken failed: {status}"))?;
      println!("{}", response.token);
    }
    PairingCommand::Connect {
      token,
      device_id,
      label,
      profile,
      json,
    } => {
      let endpoint = endpoint.ok_or_else(|| "pair connect requires --endpoint http://HOST:PORT".to_string())?;
      let endpoint_uri = endpoint.parse::<tonic::codegen::http::Uri>().map_err(|error| format!("invalid pairing endpoint: {error}"))?;
      let client_device_id = match device_id {
        Some(device_id) => device_id.clone(),
        None => random_identity()?,
      };
      tracing::info!(endpoint, service = "auv.api.daemon.v1.PairingService", method = "PairDevice", "calling bootstrap RPC");
      let enrollment = auv_api_client::protocol::grpc::clients::daemon::v1::pairing::Client::pair_device(
        endpoint_uri.clone(),
        auv_api_proto::auv::api::daemon::v1::PairDeviceRequest {
          token: token.clone(),
          device_id: client_device_id,
          label: label.clone(),
        },
      )
      .await
      .map_err(|status| format!("PairDevice failed: {status}"))?;
      tracing::debug!(endpoint, "bootstrap accepted; connecting with the returned credential");
      let client = auv_api_client::protocol::grpc::Client::connect_paired(auv_api_client::PairedConnectConfig {
        endpoint: endpoint_uri,
        device_credential: enrollment.device_credential.clone(),
      })
      .await
      .map_err(|error| format!("failed to connect to paired Device: {error}"))?;
      tracing::info!(endpoint, service = "auv.api.daemon.v1.DeviceService", method = "ListDevices", "discovering remote Device identity");
      let devices = client.devices().list_devices().await.map_err(|status| format!("ListDevices after pairing failed: {status}"))?;
      let mut remote_devices = devices.into_iter().filter(|device| device.local).collect::<Vec<_>>();
      if remote_devices.len() != 1 {
        return Err(format!("paired endpoint returned {} caller-local Devices; expected exactly one", remote_devices.len()));
      }
      let remote = remote_devices.pop().expect("one remote Device");
      let remote_id = remote
        .r#ref
        .as_ref()
        .map(|reference| reference.device_id.clone())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "paired endpoint returned a Device without an ID".to_string())?;
      let profile_store = auv::profile::ProfileStore::from_env().map_err(|error| error.to_string())?;
      let configured_profiles = match profile_store.list_devices() {
        Ok(profiles) => profiles,
        Err(auv::profile::ProfileError::Open { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.to_string()),
      };
      let existing_profile = configured_profiles
        .into_iter()
        .find(|configured| configured.device_id() == remote_id)
        .map(|configured| configured.config_profile().to_string());
      let profile_name = profile.clone().or(existing_profile).unwrap_or_else(|| {
        if remote.name.is_empty() {
          remote_id.chars().take(12).collect()
        } else {
          remote.name.clone()
        }
      });
      let device_name = remote.name.clone();
      profile_store
        .upsert(
          &profile_name,
          auv::profile::DeviceProfileInput {
            device_id: remote_id.clone(),
            device_name: device_name.clone(),
            endpoint: endpoint.to_string(),
            device_credential: enrollment.device_credential,
          },
        )
        .map_err(|error| format!("failed to save paired Device profile: {error}"))?;
      tracing::info!(path = %profile_store.config_path().display(), profile = %profile_name, "saved paired Device profile");
      if *json {
        println!(
          "{}",
          serde_json::to_string_pretty(&serde_json::json!({
            "device_id": remote_id,
            "device_name": device_name,
            "endpoint": endpoint,
            "profile": profile_name,
            "credentials_file": profile_store.config_path(),
          }))
          .map_err(|error| format!("failed to encode pairing result: {error}"))?
        );
      } else {
        let display_name = if device_name.is_empty() {
          profile_name.as_str()
        } else {
          device_name.as_str()
        };
        println!("Connected to {display_name} ({})", short_id(&remote_id));
        println!("Profile: {profile_name}");
        println!("Credentials saved in {}", profile_store.config_path().display());
      }
    }
    PairingCommand::Enable { pair_id } => {
      let store =
        PairingStore::open(store_path.clone()).map_err(|error| format!("failed to open pairing store {}: {error}", store_path.display()))?;
      store.set_enabled(pair_id, true).map_err(|error| format!("failed to enable paired device: {error}"))?;
    }
    PairingCommand::Disable { pair_id } => {
      let store =
        PairingStore::open(store_path.clone()).map_err(|error| format!("failed to open pairing store {}: {error}", store_path.display()))?;
      store.set_enabled(pair_id, false).map_err(|error| format!("failed to disable paired device: {error}"))?;
    }
  }
  Ok(())
}

#[derive(Clone)]
struct CliTracing {
  dispatch: auv_tracing::Dispatch,
  store: Arc<auv_tracing::FileTracingStore>,
}

fn build_cli_tracing(project_root: &Path, options: &TracingOptions) -> Result<CliTracing, String> {
  let store_root = resolve_store_root(project_root, options.store_root.as_ref());
  let store = Arc::new(
    auv_tracing::FileTracingStore::open(&store_root)
      .map_err(|error| format!("failed to open tracing store {}: {error}", store_root.display()))?,
  );
  let dispatch =
    auv_tracing::configure().tracing_store(store.clone()).build().map_err(|error| format!("failed to configure invoke tracing: {error}"))?;
  Ok(CliTracing { dispatch, store })
}

#[derive(serde::Serialize)]
struct InvokeFrontendLifecycle {
  frontend: &'static str,
}

impl auv_tracing::EventPayload for InvokeFrontendLifecycle {
  const NAME: &'static str = "auv.frontend.lifecycle";
  const VERSION: u32 = 1;
}

fn tracing_run_id_for_selected_context(selected: Option<&crate::plugin::ResolvedExecutionContext>) -> Result<auv_tracing::RunId, String> {
  let Some(control_run_id) = selected.and_then(|selected| selected.context.run_id.as_deref()) else {
    return Ok(auv_tracing::RunId::new());
  };
  // TODO(resource-id-migration): Remove the legacy `run_` branch after old
  // control-plane stores no longer expose prefixed UUID Run identities.
  let value = control_run_id.strip_prefix("run_").unwrap_or(control_run_id);
  let uuid = uuid::Uuid::parse_str(value)
    .map_err(|error| format!("selected Run ID {control_run_id:?} cannot be projected into the tracing Run identity: {error}"))?;
  uuid
    .to_string()
    .parse()
    .map_err(|error| format!("selected Run ID {control_run_id:?} cannot be projected into the tracing Run identity: {error}"))
}

#[cfg(test)]
#[path = "cli_frontend_test.rs"]
mod tests;
