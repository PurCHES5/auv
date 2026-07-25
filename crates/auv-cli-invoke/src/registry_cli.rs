use std::collections::BTreeMap;

use clap::error::ErrorKind;
use clap::{Arg, ArgAction, Command};

use crate::{InvokeOutputOptions, InvokeRegistry};

/// A parsed registry-backed invoke request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryInvokeRequest {
  pub command_id: String,
  pub inputs: BTreeMap<String, String>,
  pub dry_run: bool,
}

/// The outcome of parsing one registry-backed invoke command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvokeCliAction {
  Help(String),
  Invoke(RegistryInvokeRequest),
}

/// Clap-backed parsing and help for one invoke registry.
///
/// The registry remains the source of truth for command ids, descriptions,
/// and argument schemas. Frontends only provide the displayed binary prefix.
pub struct InvokeCli<'a> {
  registry: &'a InvokeRegistry,
  bin_name: &'static str,
}

impl<'a> InvokeCli<'a> {
  pub fn new(registry: &'a InvokeRegistry, bin_name: &'static str) -> Self {
    Self { registry, bin_name }
  }

  pub fn parse(&self, arguments: &[String]) -> Result<InvokeCliAction, String> {
    let mut argv = Vec::with_capacity(arguments.len() + 1);
    argv.push(self.bin_name.to_string());
    argv.extend(arguments.iter().cloned());

    let matches = match self.command().try_get_matches_from(argv) {
      Ok(matches) => matches,
      Err(error) if is_help(&error) => return Ok(InvokeCliAction::Help(error.to_string())),
      Err(error) => return Err(error.to_string()),
    };
    let (command_id, command_matches) = matches.subcommand().ok_or_else(|| "missing invoke command id".to_string())?;
    let command = self.registry.resolve(command_id).ok_or_else(|| format!("invoke registry did not resolve parsed command {command_id}"))?;
    let mut inputs = BTreeMap::new();
    for spec in command.args {
      let name = argument_name(spec.flag);
      if let Some(value) = command_matches.get_one::<String>(name) {
        inputs.insert(name.to_string(), value.clone());
      }
    }

    Ok(InvokeCliAction::Invoke(RegistryInvokeRequest {
      command_id: command_id.to_string(),
      inputs,
      dry_run: matches.get_flag("dry_run"),
    }))
  }

  pub fn render_help(&self) -> String {
    self.command().render_help().to_string()
  }

  pub fn render_command_help(&self, command_id: &str) -> Result<String, String> {
    match self.parse(&[command_id.to_string(), "--help".to_string()])? {
      InvokeCliAction::Help(help) => Ok(help),
      InvokeCliAction::Invoke(_) => Err(format!("failed to render help for command {command_id}")),
    }
  }

  fn command(&self) -> Command {
    let mut command = Command::new("invoke")
      .bin_name(self.bin_name)
      .about("Invoke a registered application operation")
      .disable_help_subcommand(true)
      .subcommand_required(true)
      .arg_required_else_help(true)
      .arg(
        Arg::new("dry_run")
          .long("dry-run")
          .global(true)
          .action(ArgAction::SetTrue)
          .help("Validate inputs without publishing operation evidence"),
      );
    for registered in self.registry.all() {
      let mut subcommand = Command::new(registered.id).about(registered.description);
      for spec in registered.args {
        let name = argument_name(spec.flag);
        subcommand = subcommand
          .arg(Arg::new(name).long(name).value_name(spec.value_name).required(spec.required).allow_hyphen_values(true).help(spec.help));
      }
      command = command.subcommand(subcommand);
    }
    command
  }
}

fn argument_name(flag: &'static str) -> &'static str {
  flag.strip_prefix("--").filter(|name| !name.is_empty()).expect("invoke argument flags must be non-empty long options")
}

fn is_help(error: &clap::Error) -> bool {
  matches!(error.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand)
}

impl From<RegistryInvokeRequest> for crate::InvokeCliParse {
  fn from(request: RegistryInvokeRequest) -> Self {
    Self::Invoke {
      command_id: request.command_id,
      target_application_id: None,
      inputs: request.inputs,
      dry_run: request.dry_run,
      output: InvokeOutputOptions::default(),
    }
  }
}
