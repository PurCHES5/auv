// File: src/cli.rs
use auv_cli_invoke::InvokeCliParse;
use auv_runtime::model::{AuvResult, ExecutionTarget, InvokeRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TracingOptions {
  pub store_root: Option<String>,
}

#[derive(Debug)]
pub enum CliCommand {
  Help,
  Version,
  PermissionCheck {
    json: bool,
  },
  ListCommandsTombstone,
  InvokeHelp {
    command_id: Option<String>,
  },
  Invoke {
    request: InvokeRequest,
    tracing: TracingOptions,
    output: auv_cli_invoke::InvokeOutputOptions,
  },
  SessionServe {
    host: String,
    port: u16,
    store_root: Option<String>,
  },
  McpServe,
  XtaskGenerateSwiftBridge,
}

pub fn parse_cli(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.is_empty() {
    return Ok(CliCommand::Help);
  }

  if root_version_requested(arguments) {
    return Ok(CliCommand::Version);
  }

  match arguments[0].as_str() {
    "help" | "--help" | "-h" => Ok(CliCommand::Help),
    "--version" | "-V" => Err("usage: auv --version".to_string()),
    "doctor" => parse_permission_check(arguments),
    "permissions" => parse_permissions(arguments),
    "--xtask" => parse_xtask(arguments),
    "list-commands" => Ok(CliCommand::ListCommandsTombstone),
    "inspect" => Err("`auv inspect` has been retired; the replacement inspector read-side is intentionally deferred".to_string()),
    "session" => parse_session(arguments),
    "mcp" => parse_mcp(arguments),
    "invoke" => parse_invoke(arguments),
    "skill" => Err("skill commands have been removed; use app-local Rust commands instead".to_string()),
    other => Err(format!("unknown subcommand {other}; use `help` to see supported commands")),
  }
}

/// Returns whether root `auv` can print its version before creating an async runtime.
pub fn root_version_requested(arguments: &[String]) -> bool {
  matches!(arguments, [flag] if matches!(flag.as_str(), "--version" | "-V"))
}

fn parse_xtask(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() != 2 {
    return Err("usage: auv --xtask generate-swift-bridge".to_string());
  }

  match arguments[1].as_str() {
    "generate-swift-bridge" => Ok(CliCommand::XtaskGenerateSwiftBridge),
    other => Err(format!("unknown xtask {other}; supported xtasks: generate-swift-bridge")),
  }
}

pub fn help_text() -> String {
  String::from(
    "\
  auv prototype

USAGE
  auv --version
  auv doctor [--json]
  auv permissions check [--json]
  auv invoke <command-id> [--dry-run] [--target <application-id>] [--label <text>] [--store-root <path>]
  auv session serve [--host <host>] [--port <port>] [--store-root <path>]
  auv mcp serve

NOTES
  - Names are provisional and reflect the current phase-0/1 runtime skeleton.
  - The CLI is a thin frontend over the library runtime in src/lib.rs.
  - `invoke --help` is the discovery surface for canonical invoke commands in the current C1 scaffold.
  - `list-commands` has been retired; use `auv invoke --help` instead.
  - Live invoke presentation is enabled by default where the local driver has an overlay adapter; pass `--no-overlay` to disable it. The `overlay.*` debug command descriptors remain unregistered until their session handlers land.
  - `window.captureAxTree`, `input.focusText`, and `input.pressButton` accept `--reveal_shortcut cmd+f`-style hints when an app hides the target UI until a keyboard shortcut reveals it.
  - `--reveal_settle_ms <millis>` can be used to make the reveal step explicit instead of depending on hard-coded timing assumptions.
  - `input.typeText` supports `--replace_existing true`, `--submit_key return`, and `--submit_settle_ms 800` for repeatable text-entry flows.
  - `input.key` supports both special keys like `Return` and shortcuts like `cmd+f`, with optional `--settle_ms`.
  - `input.clickWindowPoint` accepts either `--offset_x/--offset_y` or `--relative_x/--relative_y` against the target window bounds.
  - `input.teachClick` captures a target window before a human-taught click, opens a small Ready prompt, records the next click as global/window-local coordinates, then captures follow-up frames at `--first_after_ms` and `--second_after_ms` (defaults 150/250).
  - `screen.findText` and `screen.clickText` use macOS Vision OCR over a captured screenshot and operate in screenshot-pixel anchors projected back to logical points.
  - `screen.waitForText` polls that same OCR path until a filtered anchor appears or the timeout expires; use it when result-page readiness is the real problem instead of guessing longer sleeps.
  - `screen.findRows`, `screen.waitForRows`, and `screen.clickRow` treat OCR observations as grouped visible rows, which is the current fallback direction when exact text anchors are visually present but not OCR-reliable.
  - `screen.findImageText` runs the same OCR matching over an existing image artifact, which is useful for verifying captured evidence without recapturing the live desktop.
  - `mediaControl.nowPlaying` prefers AX tree matching for player-title verification, which is the current direction for native playback disambiguation.
  - `window.verifyText` is the generic AX-tree text verification contract for native apps with reliable text-bearing nodes.
  - `screen.clickText` supports `--match_index` and `--click_count` when the query resolves to multiple OCR anchors.
",
  )
}

pub fn version_text() -> String {
  format!("auv {}\n", env!("CARGO_PKG_VERSION"))
}

fn parse_permission_check(arguments: &[String]) -> AuvResult<CliCommand> {
  let mut json = false;
  for argument in arguments.iter().skip(1) {
    match argument.as_str() {
      "--json" => json = true,
      other => {
        return Err(format!("unknown doctor option {other}; usage: auv doctor [--json]"));
      }
    }
  }

  Ok(CliCommand::PermissionCheck { json })
}

fn parse_permissions(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() < 2 {
    return Err("usage: auv permissions check [--json]".to_string());
  }

  match arguments[1].as_str() {
    "check" => {
      let mut normalized = vec!["doctor".to_string()];
      normalized.extend(arguments.iter().skip(2).cloned());
      parse_permission_check(&normalized)
    }
    other => Err(format!("unknown permissions subcommand {other}; usage: auv permissions check [--json]")),
  }
}

fn parse_tracing_option(argument: &str, value: Option<&String>, tracing: &mut TracingOptions) -> AuvResult<Option<usize>> {
  match argument {
    "--store-root" => {
      let value = value.ok_or_else(|| "--store-root requires a value".to_string())?;
      tracing.store_root = Some(value.clone());
      Ok(Some(2))
    }
    _ => Ok(None),
  }
}

fn parse_invoke(arguments: &[String]) -> AuvResult<CliCommand> {
  let mut tracing = TracingOptions::default();
  let mut invoke_arguments = Vec::with_capacity(arguments.len());
  let mut index = 0;

  if let Some(subcommand) = arguments.first() {
    invoke_arguments.push(subcommand.clone());
    index = 1;
  }

  if let Some(command_or_help) = arguments.get(index) {
    invoke_arguments.push(command_or_help.clone());
    index += 1;
  }

  while index < arguments.len() {
    let argument = arguments[index].as_str();
    if let Some(consumed) = parse_tracing_option(argument, arguments.get(index + 1), &mut tracing)? {
      index += consumed;
      continue;
    }

    invoke_arguments.push(arguments[index].clone());
    if !auv_cli_invoke::invoke_argument_consumes_value(argument) {
      index += 1;
      continue;
    }

    if let Some(value) = arguments.get(index + 1) {
      invoke_arguments.push(value.clone());
      index += 2;
      continue;
    }

    index += 1;
  }

  match auv_cli_invoke::parse_invoke_args(&invoke_arguments)? {
    InvokeCliParse::Help { command_id } => Ok(CliCommand::InvokeHelp { command_id }),
    InvokeCliParse::Invoke {
      command_id,
      target_application_id,
      inputs,
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
      tracing,
      output,
    }),
  }
}

fn parse_mcp(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() != 2 || arguments[1].as_str() != "serve" {
    return Err("usage: auv mcp serve".to_string());
  }
  Ok(CliCommand::McpServe)
}

fn parse_session(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() < 2 {
    return Err("usage: auv session serve [--host <host>] [--port <port>] [--store-root <path>]".to_string());
  }
  if arguments[1].as_str() != "serve" {
    return Err("usage: auv session serve [--host <host>] [--port <port>] [--store-root <path>]".to_string());
  }
  parse_session_serve(arguments)
}

fn parse_session_serve(arguments: &[String]) -> AuvResult<CliCommand> {
  let mut host = auv_runtime::api::session_service::transport::DEFAULT_SESSION_API_HOST.to_string();
  let mut port = auv_runtime::api::session_service::transport::DEFAULT_SESSION_API_PORT;
  let mut store_root = None;
  let mut index = 2;
  while index < arguments.len() {
    match arguments[index].as_str() {
      "--host" => {
        if index + 1 >= arguments.len() {
          return Err("--host requires a value".to_string());
        }
        host = arguments[index + 1].clone();
        index += 2;
      }
      "--port" => {
        if index + 1 >= arguments.len() {
          return Err("--port requires a value".to_string());
        }
        port = arguments[index + 1].parse::<u16>().map_err(|error| format!("invalid --port value: {error}"))?;
        index += 2;
      }
      "--store-root" => {
        if index + 1 >= arguments.len() {
          return Err("--store-root requires a value".to_string());
        }
        store_root = Some(arguments[index + 1].clone());
        index += 2;
      }
      other => {
        return Err(format!("unexpected session-serve argument {other}"));
      }
    }
  }

  Ok(CliCommand::SessionServe {
    host,
    port,
    store_root,
  })
}
