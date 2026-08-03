use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::commands::pairing::PairingArgs;

#[derive(Clone, Debug, Args)]
#[command(
  after_long_help = "Examples:\n  # List local and paired Devices\n  auv devices list\n\n  # Inspect one Device by its stable ID\n  auv devices get <DEVICE_ID>\n\n  # Learn the two-machine enrollment flow\n  auv devices pair --help\n\n  # Run the same typed operation on a paired Device\n  auv --device <NAME> invoke display.list"
)]
pub struct DevicesArgs {
  #[command(subcommand)]
  pub command: DevicesCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum DevicesCommand {
  /// List Devices visible through the selected daemon.
  #[command(alias = "ls")]
  List(DeviceListArgs),
  /// Get one Device by stable ID.
  Get(DeviceGetArgs),
  /// Establish or administer paired Device trust.
  #[command(
    long_about = "Pairing is a two-machine enrollment flow. The daemon host creates a short, one-time bootstrap token locally. The client consumes that token once, receives an opaque Device credential, and saves it in a named local profile.\n\nRegistration is deliberately local to the daemon host: a remote unauthenticated caller cannot create pairing tokens. After enrollment, normal AUV commands select the paired Device and reuse the saved credential automatically."
  )]
  Pair(PairingArgs),
  /// Remove a paired Device trust relationship.
  Unpair(DeviceTrustArgs),
  /// Enable a paired Device trust relationship.
  Enable(DeviceTrustArgs),
  /// Disable a paired Device trust relationship without deleting its history.
  Disable(DeviceTrustArgs),
  /// Manage configured paired Device profiles (provisional).
  Profiles(DeviceProfilesArgs),
}

#[derive(Clone, Debug, Args)]
pub struct DeviceTrustArgs {
  /// Stable Device ID or human-facing Device name.
  pub device: String,
  /// Durable pairing-store path used by the remote API server.
  #[arg(long, value_name = "PATH")]
  pub store: Option<PathBuf>,
}

#[derive(Clone, Debug, Args)]
pub struct DeviceProfilesArgs {
  #[command(subcommand)]
  pub command: DeviceProfilesCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum DeviceProfilesCommand {
  List(ProfileOutputArgs),
  Get(ProfileGetArgs),
  Create(ProfileWriteArgs),
  Update(ProfileWriteArgs),
  Delete(ProfileDeleteArgs),
}

#[derive(Clone, Debug, Args)]
pub struct ProfileOutputArgs {
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct ProfileGetArgs {
  pub name: String,
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct ProfileDeleteArgs {
  pub name: String,
}

#[derive(Clone, Args)]
pub struct ProfileWriteArgs {
  pub name: String,
  #[arg(long)]
  pub device_id: String,
  #[arg(long)]
  pub device_name: String,
  #[arg(long)]
  pub endpoint: String,
  /// Opaque credential saved by `auv devices pair connect`.
  #[arg(long)]
  pub device_credential: String,
}

impl std::fmt::Debug for ProfileWriteArgs {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("ProfileWriteArgs")
      .field("name", &self.name)
      .field("device_id", &self.device_id)
      .field("device_name", &self.device_name)
      .field("endpoint", &self.endpoint)
      .field("device_credential", &"[REDACTED]")
      .finish()
  }
}

#[derive(Clone, Debug, Args)]
pub struct DeviceListArgs {
  /// Override daemon discovery with an explicit endpoint.
  #[arg(long, value_name = "URI")]
  pub endpoint: Option<String>,
  /// Render machine-readable JSON.
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct DeviceGetArgs {
  /// Stable Device ID.
  pub device_id: String,
  /// Override daemon discovery with an explicit endpoint.
  #[arg(long, value_name = "URI")]
  pub endpoint: Option<String>,
  /// Render machine-readable JSON.
  #[arg(long)]
  pub json: bool,
}
