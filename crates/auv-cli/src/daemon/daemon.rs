use std::io::Write;
use std::path::{Path, PathBuf};

use super::discovery;

pub(crate) fn default_local_listener(discovery_file: Option<&Path>) -> Result<String, String> {
  #[cfg(unix)]
  {
    let descriptor = match discovery_file {
      Some(path) => path.to_path_buf(),
      None => discovery::default_path().map_err(|error| error.to_string())?,
    };
    let parent = descriptor.parent().ok_or_else(|| format!("daemon descriptor path has no parent: {}", descriptor.display()))?;
    let socket = parent.join("auv.sock");
    Ok(format!("unix://{}", socket.display()))
  }
  #[cfg(not(unix))]
  Ok(format!("http://{}:{}", auv_api_server::server::DEFAULT_API_HOST, auv_api_server::server::DEFAULT_API_PORT))
}

pub(crate) fn parse_listener(listener: &str, paired_tcp: bool) -> Result<auv_api_server::server::ListenEndpoint, String> {
  if let Some(authority) = listener.strip_prefix("http://") {
    let address = authority
      .parse::<std::net::SocketAddr>()
      .map_err(|error| format!("invalid --listen URI {listener:?}; expected http://IP:PORT: {error}"))?;
    if paired_tcp || !address.ip().is_loopback() {
      return Ok(auv_api_server::server::ListenEndpoint::Remote {
        host: address.ip().to_string(),
        port: address.port(),
      });
    }
  }
  let endpoint = listener.parse::<auv_api_client::ConnectEndpoint>().map_err(|error| format!("invalid --listen URI: {error}"))?;
  match endpoint {
    auv_api_client::ConnectEndpoint::Tcp(uri) => {
      let host = uri.host().ok_or_else(|| "--listen TCP URI omitted host".to_string())?.to_string();
      let port = uri.port_u16().unwrap_or(80);
      Ok(auv_api_server::server::ListenEndpoint::Tcp { host, port })
    }
    #[cfg(unix)]
    auv_api_client::ConnectEndpoint::Unix(path) => Ok(auv_api_server::server::ListenEndpoint::Unix { path }),
  }
}

pub(crate) async fn serve_foreground(
  listeners: Vec<auv_api_server::server::ListenEndpoint>,
  store_root: PathBuf,
  pairing_store: Option<PathBuf>,
  discovery_file: Option<PathBuf>,
  no_discovery: bool,
  daemon_idle_timeout: Option<std::time::Duration>,
  runner_provider_paths: &[PathBuf],
) -> Result<i32, String> {
  let runner_providers = runner_provider_paths
    .iter()
    .map(|path| {
      let path = if path.is_absolute() {
        path.clone()
      } else {
        std::env::current_dir().map_err(|error| format!("failed to resolve --runner-provider {}: {error}", path.display()))?.join(path)
      };
      auv_api_server::runner_provider::RunnerProviderConfig::load_json(&path)
        .map_err(|error| format!("failed to load --runner-provider {}: {error}", path.display()))
    })
    .collect::<Result<Vec<_>, _>>()?;
  let mut listeners = listeners.into_iter();
  let listen = listeners.next().ok_or_else(|| "auv serve requires at least one listener".to_string())?;
  let first_party_runners = first_party_runner_runtimes(&store_root)?;
  let bound = auv_api_server::server::Server::bind(auv_api_server::server::ServerConfig {
    listen,
    additional_listeners: listeners.collect(),
    store_root,
    pairing_store,
    daemon_idle_timeout,
    runner_providers,
    first_party_runners,
  })
  .await?;
  let _published_descriptor = if no_discovery {
    None
  } else {
    let path = match discovery_file {
      Some(path) => path,
      None => discovery::default_path().map_err(|error| error.to_string())?,
    };
    match bound.discovery_endpoint() {
      Some(local_endpoint) => Some(discovery::PublishedDescriptor::publish(path, local_endpoint.to_string())?),
      None => None,
    }
  };
  for endpoint in bound.endpoints() {
    println!("auv serve: {endpoint}");
  }
  std::io::stdout().flush().map_err(|error| format!("failed to flush daemon readiness line: {error}"))?;
  let shutdown = tokio_util::sync::CancellationToken::new();
  let signal_shutdown = shutdown.clone();
  tokio::spawn(async move {
    if tokio::signal::ctrl_c().await.is_ok() {
      signal_shutdown.cancel();
    }
  });
  bound.serve(shutdown).await?;
  Ok(0)
}

fn first_party_runner_runtimes(store_root: &std::path::Path) -> Result<auv_api_server::runner_provider::FirstPartyRunnerRuntimes, String> {
  #[cfg(unix)]
  {
    use std::collections::BTreeMap;

    use auv_api_server::runner_provider::{ExecutableRunnerRuntime, RunnerRuntime};

    let executable = std::env::current_exe().map_err(|error| format!("failed to resolve the auv executable for Runner hosting: {error}"))?;
    let runtime = |role: &str, environment: BTreeMap<String, String>| {
      RunnerRuntime::Executable(ExecutableRunnerRuntime {
        executable: executable.clone(),
        arguments: vec![
          crate::runner::INTERNAL_SENTINEL.to_string(),
          role.to_string(),
        ],
        working_directory: None,
        environment,
      })
    };
    let runner_state_root = store_root.join("runner-state").join("auv.core.local");
    let runner_state_root =
      runner_state_root.to_str().ok_or_else(|| format!("local Runner state path is not valid UTF-8: {}", runner_state_root.display()))?;
    let local_environment = BTreeMap::from([(crate::runner::STATE_ROOT_ENV.to_string(), runner_state_root.to_string())]);
    Ok(auv_api_server::runner_provider::FirstPartyRunnerRuntimes {
      local_driver: Some(runtime(crate::runner::LOCAL_DRIVER_ROLE, local_environment)),
    })
  }
  #[cfg(not(unix))]
  {
    // TODO(first-party-runner-windows-host): publish built-in runtimes after
    // inherited named-pipe transport replaces the Unix descriptor contract.
    Ok(Default::default())
  }
}

#[cfg(test)]
#[path = "daemon_test.rs"]
mod tests;
