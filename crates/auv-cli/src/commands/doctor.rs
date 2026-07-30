use clap::Args;

/// Inspect the permissions needed by local desktop automation.
#[derive(Clone, Debug, Args)]
#[command(
  after_long_help = "Examples:\n  # Print a human-readable permission report\n  auv doctor\n\n  # Print a machine-readable permission report\n  auv doctor --json"
)]
pub struct DoctorArgs {
  /// Render the permission report as JSON.
  #[arg(long)]
  pub json: bool,
}
