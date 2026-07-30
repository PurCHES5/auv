use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OutputMode {
  Human,
  Json,
  JsonFile(PathBuf),
}

#[derive(Clone, Debug, Default, Args)]
pub(super) struct OutputArgs {
  /// Print the structured command result as JSON on stdout.
  #[arg(long = "json")]
  json: bool,
  /// Write the structured command result as JSON to this file.
  #[arg(long = "json-out")]
  json_out: Option<PathBuf>,
}

impl OutputArgs {
  pub fn mode(self) -> OutputMode {
    match self.json_out {
      Some(path) => OutputMode::JsonFile(path),
      None if self.json => OutputMode::Json,
      None => OutputMode::Human,
    }
  }

  pub fn mode_with_json_alias(self, alias: bool) -> OutputMode {
    match self.json_out {
      Some(path) => OutputMode::JsonFile(path),
      None if self.json || alias => OutputMode::Json,
      None => OutputMode::Human,
    }
  }
}

/// Emit one command result according to the shared CLI output contract.
pub(super) fn emit<T>(mode: &OutputMode, json_value: &T, human: impl FnOnce() -> String) -> ExitCode
where
  T: Serialize + ?Sized,
{
  match mode {
    OutputMode::Human => {
      println!("{}", human());
      ExitCode::SUCCESS
    }
    OutputMode::Json => match serde_json::to_string_pretty(json_value) {
      Ok(json) => {
        println!("{json}");
        ExitCode::SUCCESS
      }
      Err(error) => {
        eprintln!("encode failed: {error}");
        ExitCode::from(1)
      }
    },
    OutputMode::JsonFile(path) => {
      let json = match serde_json::to_string_pretty(json_value) {
        Ok(json) => json,
        Err(error) => {
          eprintln!("encode failed: {error}");
          return ExitCode::from(1);
        }
      };
      if let Err(error) = std::fs::write(path, json) {
        eprintln!("failed to write {}: {error}", path.display());
        return ExitCode::from(1);
      }
      ExitCode::SUCCESS
    }
  }
}
