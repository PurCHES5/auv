use clap::{Args, Subcommand};

/// Inspect operating-system permissions used by AUV drivers.
#[derive(Clone, Debug, Args)]
pub struct PermissionsArgs {
  #[command(subcommand)]
  pub command: PermissionsCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum PermissionsCommand {
  /// Check permissions needed by local desktop automation.
  Check(PermissionsCheckArgs),
}

/// Check permissions needed by local desktop automation.
#[derive(Clone, Debug, Args)]
#[command(
  after_long_help = "Examples:\n  # Print a human-readable permission report\n  auv permissions check\n\n  # Print a machine-readable permission report\n  auv permissions check --json"
)]
pub struct PermissionsCheckArgs {
  /// Render the permission report as JSON.
  #[arg(long)]
  pub json: bool,
}
