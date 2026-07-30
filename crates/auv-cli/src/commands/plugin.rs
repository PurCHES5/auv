use clap::{Args, Subcommand};

/// Inspect external auv-* command plugins visible on PATH.
#[derive(Clone, Debug, Args)]
pub struct PluginArgs {
  #[command(subcommand)]
  pub command: PluginCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum PluginCommand {
  /// List external auv-* executables visible on PATH.
  List,
}
