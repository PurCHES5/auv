//! Typed root command tree for the core `auv` frontend.

use std::ffi::OsString;
use std::path::PathBuf;

use auv_cli_invoke::{ExecutionTarget, InvokeCliParse, InvokeRequest};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};

use crate::commands::api_server::{ApiServerArgs, ApiServerCommand};
use crate::commands::devices::{DeviceProfilesCommand, DevicesArgs, DevicesCommand};
use crate::commands::doctor::DoctorArgs;
use crate::commands::invoke::InvokeArgs;
use crate::commands::mcp::{McpArgs, McpCommand};
use crate::commands::pairing::PairingCommand;
use crate::commands::plugin::{PluginArgs, PluginCommand};
use crate::commands::run::{RunArgs, RunCommand};
use crate::commands::runner::{LifecycleArg, RunnerArgs, RunnerCommand};
use crate::commands::serve::ServeArgs;

type AuvResult<T> = Result<T, String>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TracingOptions {
  pub store_root: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParentContextOptions {
  pub device_name: Option<String>,
  pub device_id: Option<String>,
  pub run_id: Option<String>,
}

#[derive(Debug)]
pub enum CliCommand {
  Help(String),
  Version,
  PermissionCheck {
    json: bool,
  },
  InvokeHelp {
    command_id: Option<String>,
  },
  Invoke {
    request: InvokeRequest,
    typed_args: auv_cli_invoke::TypedInvokeArgs,
    tracing: TracingOptions,
    output: auv_cli_invoke::InvokeOutputOptions,
    parent_context: ParentContextOptions,
  },
  ApiServerServe {
    host: String,
    port: u16,
    remote_listen: Option<String>,
    pairing_store: Option<PathBuf>,
    #[cfg(unix)]
    unix_socket: Option<PathBuf>,
    store_root: Option<PathBuf>,
    discovery_file: Option<PathBuf>,
    no_discovery: bool,
    daemon_idle_timeout: Option<std::time::Duration>,
    runner_providers: Vec<PathBuf>,
  },
  Serve {
    listeners: Vec<String>,
    pairing_store: Option<PathBuf>,
    store_root: Option<PathBuf>,
    discovery_file: Option<PathBuf>,
    no_discovery: bool,
    daemon_idle_timeout: Option<std::time::Duration>,
    runner_providers: Vec<PathBuf>,
  },
  DeviceList {
    endpoint: Option<String>,
    json: bool,
    parent_context: ParentContextOptions,
  },
  DeviceGet {
    endpoint: Option<String>,
    device_id: String,
    json: bool,
    parent_context: ParentContextOptions,
  },
  DeviceProfiles {
    command: DeviceProfilesCommand,
  },
  DeviceTrust {
    store: Option<PathBuf>,
    device: String,
    action: DeviceTrustAction,
  },
  RunnerCreate {
    endpoint: Option<String>,
    runner_class: String,
    lifecycle: i32,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunnerList {
    endpoint: Option<String>,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunnerClassList {
    endpoint: Option<String>,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunnerGet {
    endpoint: Option<String>,
    runner_id: String,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunnerStop {
    endpoint: Option<String>,
    runner_id: String,
    timeout: Option<u64>,
    force: bool,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunCreate {
    endpoint: Option<String>,
    device_ids: Vec<String>,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunList {
    endpoint: Option<String>,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunGet {
    endpoint: Option<String>,
    run_id: String,
    json: bool,
    parent_context: ParentContextOptions,
  },
  RunStop {
    endpoint: Option<String>,
    run_id: String,
    outcome: i32,
    json: bool,
    parent_context: ParentContextOptions,
  },
  Pairing {
    store: Option<PathBuf>,
    endpoint: Option<String>,
    command: PairingCommand,
  },
  McpServe,
  PluginList,
  External {
    command_name: OsString,
    arguments: Vec<OsString>,
    parent_context: ParentContextOptions,
  },
  XtaskGenerateSwiftBridge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceTrustAction {
  Unpair,
  Enable,
  Disable,
}

#[derive(Debug, Parser)]
#[command(
  name = "auv",
  version,
  about = "Invoke and inspect core computer-use capabilities",
  long_about = "AUV turns computer-use operations into command-like, inspectable, and recorded runs.\n\nThe root CLI owns core invoke, doctor, API-server, Device/Run/Runner control, and MCP frontends. Installed auv-* executables extend it with application-owned commands.",
  after_long_help = "Examples:\n  # Inspect available core invoke commands\n  auv invoke --help\n\n  # Diagnose local automation readiness\n  auv doctor\n\n  # Run an installed application plugin\n  auv balatro --help\n\nUse `auv plugin list` to inspect external commands visible on PATH."
)]
struct RootArgs {
  /// Increase connection and RPC diagnostics. Repeat for more detail.
  #[arg(short, long, action = clap::ArgAction::Count, global = true)]
  verbose: u8,

  /// Run a hidden repository development task.
  #[arg(long, value_enum, hide = true)]
  xtask: Option<Xtask>,

  /// Select a Device by its human-facing name for this invocation.
  #[arg(long, value_name = "NAME")]
  device: Option<String>,

  /// Select a Device by its stable ID for this invocation.
  #[arg(long, value_name = "ID")]
  device_id: Option<String>,

  /// Append this invocation to an existing Run.
  #[arg(long, value_name = "ID")]
  run: Option<String>,

  #[command(subcommand)]
  command: Option<RootCommand>,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
  /// Inspect local automation permissions and environment readiness.
  Doctor(DoctorArgs),

  /// Invoke one core computer-use capability and record its run.
  Invoke(InvokeArgs),

  /// Run the AUV API server.
  #[command(hide = true)]
  ApiServer(ApiServerArgs),

  /// Run the AUV daemon in the foreground.
  Serve(ServeArgs),

  /// Inspect Devices visible through an AUV daemon.
  #[command(
    long_about = "Devices are AUV execution targets exposed by a daemon. A local daemon publishes this machine as a Device; pairing adds another daemon as a remotely selectable Device.\n\n`auv devices list` combines the local daemon with saved paired profiles and reports whether each target is online. Pairing credentials stay in the local profile store and are reused automatically by later commands."
  )]
  Devices(DevicesArgs),

  /// Create and inspect daemon-owned Runners.
  #[command(visible_alias = "runners")]
  Runner(RunnerArgs),

  /// Create and inspect Run correlation scopes.
  Run(RunArgs),

  /// Expose core AUV capabilities through MCP.
  Mcp(McpArgs),

  /// Inspect external auv-* command plugins visible on PATH.
  Plugin(PluginArgs),

  #[command(external_subcommand)]
  External(Vec<OsString>),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Xtask {
  GenerateSwiftBridge,
}

pub fn parse_cli(arguments: &[String]) -> AuvResult<CliCommand> {
  parse_cli_os(arguments.iter().map(OsString::from))
}

pub fn parse_cli_os(arguments: impl IntoIterator<Item = OsString>) -> AuvResult<CliCommand> {
  let arguments = arguments.into_iter().collect::<Vec<_>>();
  if arguments.is_empty() {
    return Ok(CliCommand::Help(help_text()));
  }

  let mut argv = Vec::with_capacity(arguments.len() + 1);
  argv.push(OsString::from("auv"));
  argv.extend(arguments);
  let parsed = match RootArgs::try_parse_from(argv) {
    Ok(parsed) => parsed,
    Err(error) => {
      return match error.kind() {
        ErrorKind::DisplayHelp => Ok(CliCommand::Help(error.to_string())),
        ErrorKind::DisplayVersion => Ok(CliCommand::Version),
        _ => Err(error.to_string()),
      };
    }
  };

  let parent_context = ParentContextOptions {
    device_name: parsed.device,
    device_id: parsed.device_id,
    run_id: parsed.run,
  };
  let _verbosity = parsed.verbose;

  if let Some(xtask) = parsed.xtask {
    return match xtask {
      Xtask::GenerateSwiftBridge => Ok(CliCommand::XtaskGenerateSwiftBridge),
    };
  }

  match parsed.command {
    None => Ok(CliCommand::Help(help_text())),
    Some(RootCommand::Doctor(args)) => Ok(CliCommand::PermissionCheck { json: args.json }),
    Some(RootCommand::Invoke(args)) => parse_invoke(args.arguments, parent_context),
    Some(RootCommand::ApiServer(args)) => match args.command {
      ApiServerCommand::Serve(args) => {
        if args.remote_listen.is_some() && args.pairing_store.is_none() {
          return Err("--remote-listen requires --pairing-store".to_string());
        }
        #[cfg(unix)]
        if args.remote_listen.is_some() && args.unix_socket.is_some() {
          return Err("--remote-listen conflicts with --unix-socket".to_string());
        }
        Ok(CliCommand::ApiServerServe {
          host: args.host,
          port: args.port,
          remote_listen: args.remote_listen,
          pairing_store: args.pairing_store,
          #[cfg(unix)]
          unix_socket: args.unix_socket,
          store_root: args.store_root,
          discovery_file: args.discovery_file,
          no_discovery: args.no_discovery,
          daemon_idle_timeout: args.daemon_idle_timeout.map(std::time::Duration::from_secs),
          runner_providers: args.runner_providers,
        })
      }
    },
    Some(RootCommand::Serve(args)) => Ok(CliCommand::Serve {
      listeners: args.listeners,
      pairing_store: args.pairing_store,
      store_root: args.store_root,
      discovery_file: args.discovery_file,
      no_discovery: args.no_discovery,
      daemon_idle_timeout: args.daemon_idle_timeout.map(std::time::Duration::from_secs),
      runner_providers: args.runner_providers,
    }),
    Some(RootCommand::Devices(args)) => match args.command {
      DevicesCommand::List(args) => Ok(CliCommand::DeviceList {
        endpoint: args.endpoint,
        json: args.json,
        parent_context,
      }),
      DevicesCommand::Get(args) => Ok(CliCommand::DeviceGet {
        endpoint: args.endpoint,
        device_id: args.device_id,
        json: args.json,
        parent_context,
      }),
      DevicesCommand::Pair(args) => Ok(CliCommand::Pairing {
        store: args.store,
        endpoint: args.endpoint,
        command: args.command,
      }),
      DevicesCommand::Unpair(args) => Ok(CliCommand::DeviceTrust {
        store: args.store,
        device: args.device,
        action: DeviceTrustAction::Unpair,
      }),
      DevicesCommand::Enable(args) => Ok(CliCommand::DeviceTrust {
        store: args.store,
        device: args.device,
        action: DeviceTrustAction::Enable,
      }),
      DevicesCommand::Disable(args) => Ok(CliCommand::DeviceTrust {
        store: args.store,
        device: args.device,
        action: DeviceTrustAction::Disable,
      }),
      DevicesCommand::Profiles(args) => Ok(CliCommand::DeviceProfiles {
        command: args.command,
      }),
    },
    Some(RootCommand::Runner(args)) => match args.command {
      RunnerCommand::Create(args) => Ok(CliCommand::RunnerCreate {
        endpoint: args.endpoint,
        runner_class: args.runner_class,
        lifecycle: match args.lifecycle {
          LifecycleArg::Ephemeral => auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::Ephemeral as i32,
          LifecycleArg::UnlessIdle => auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::UnlessIdle as i32,
          LifecycleArg::UnlessShutdown => auv_api_proto::auv::api::daemon::v1::RunnerLifecycle::UnlessShutdown as i32,
        },
        json: args.json,
        parent_context,
      }),
      RunnerCommand::List(args) => Ok(CliCommand::RunnerList {
        endpoint: args.endpoint,
        json: args.json,
        parent_context,
      }),
      RunnerCommand::Classes(args) => Ok(CliCommand::RunnerClassList {
        endpoint: args.endpoint,
        json: args.json,
        parent_context,
      }),
      RunnerCommand::Get(args) => Ok(CliCommand::RunnerGet {
        endpoint: args.endpoint,
        runner_id: args.runner_id,
        json: args.json,
        parent_context,
      }),
      RunnerCommand::Stop(args) => Ok(CliCommand::RunnerStop {
        endpoint: args.endpoint,
        runner_id: args.runner_id,
        timeout: args.timeout,
        force: args.force,
        json: args.json,
        parent_context,
      }),
    },
    Some(RootCommand::Run(args)) => match args.command {
      RunCommand::Create(args) => Ok(CliCommand::RunCreate {
        endpoint: args.endpoint,
        device_ids: args.device_ids,
        json: args.json,
        parent_context,
      }),
      RunCommand::List(args) => Ok(CliCommand::RunList {
        endpoint: args.endpoint,
        json: args.json,
        parent_context,
      }),
      RunCommand::Get(args) => Ok(CliCommand::RunGet {
        endpoint: args.endpoint,
        run_id: args.run_id,
        json: args.json,
        parent_context,
      }),
      RunCommand::Stop(args) => Ok(CliCommand::RunStop {
        endpoint: args.endpoint,
        run_id: args.run_id,
        outcome: match args.outcome {
          crate::commands::run::RunOutcomeArg::Succeeded => auv_api_proto::auv::api::daemon::v1::RunOutcome::Succeeded as i32,
          crate::commands::run::RunOutcomeArg::Failed => auv_api_proto::auv::api::daemon::v1::RunOutcome::Failed as i32,
          crate::commands::run::RunOutcomeArg::Canceled => auv_api_proto::auv::api::daemon::v1::RunOutcome::Canceled as i32,
        },
        json: args.json,
        parent_context,
      }),
    },
    Some(RootCommand::Mcp(args)) => match args.command {
      McpCommand::Serve => Ok(CliCommand::McpServe),
    },
    Some(RootCommand::Plugin(args)) => match args.command {
      PluginCommand::List => Ok(CliCommand::PluginList),
    },
    Some(RootCommand::External(mut arguments)) => {
      let command_name = arguments.remove(0);
      Ok(CliCommand::External {
        command_name,
        arguments,
        parent_context,
      })
    }
  }
}

pub fn help_text() -> String {
  RootArgs::command().render_long_help().to_string()
}

pub fn version_text() -> String {
  format!("auv {}\n", env!("CARGO_PKG_VERSION"))
}

fn parse_invoke(arguments: Vec<String>, parent_context: ParentContextOptions) -> AuvResult<CliCommand> {
  let mut invoke_arguments = vec!["invoke".to_string()];
  invoke_arguments.extend(arguments);

  match auv_cli_invoke::parse_invoke_args(&invoke_arguments)? {
    InvokeCliParse::Help { command_id } => Ok(CliCommand::InvokeHelp { command_id }),
    InvokeCliParse::Invoke {
      command_id,
      target_application_id,
      inputs,
      typed_args,
      store_root,
      dry_run,
      output,
    } => Ok(CliCommand::Invoke {
      request: InvokeRequest {
        command_id,
        target: ExecutionTarget {
          application_id: target_application_id,
        },
        inputs,
        dry_run,
      },
      typed_args,
      tracing: TracingOptions { store_root },
      output,
      parent_context,
    }),
  }
}

#[cfg(test)]
#[path = "cli_test.rs"]
mod tests;
