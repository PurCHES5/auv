use std::path::PathBuf;

use clap::{Args, Subcommand};

/// Create and consume short paired-Device enrollment tokens.
#[derive(Clone, Debug, Args)]
#[command(
  after_long_help = "Examples:\n  # On the daemon host: create a one-time bootstrap token\n  auv devices pair create-token\n\n  # On the client machine: consume the token and save a profile\n  auv devices pair --endpoint http://HOST:9847 connect --token <TOKEN> --label \"MacBook\" --profile node1\n\n  # Confirm that the paired Device is online\n  auv devices list\n\n  # Invoke a typed operation through the paired daemon\n  auv --device <NAME> invoke display.list"
)]
pub struct PairingArgs {
  /// Daemon endpoint. Create-token defaults to local discovery; connect must
  /// name the remote http:// endpoint that issued the token.
  #[arg(long, global = true, value_name = "URI")]
  pub endpoint: Option<String>,

  /// Durable pairing-store path used by the remote API server.
  #[arg(long, global = true, value_name = "PATH")]
  pub store: Option<PathBuf>,

  #[command(subcommand)]
  pub command: PairingCommand,
}

#[derive(Clone, Subcommand)]
pub enum PairingCommand {
  /// Create a cryptographically random bootstrap token and display it once.
  CreateToken {
    /// Optional token lifetime in seconds. Omit for no expiry.
    #[arg(long, value_name = "SECONDS")]
    ttl: Option<u64>,
  },
  /// Connect to a Device and save its credential as a local profile.
  Connect {
    #[arg(long)]
    token: String,
    /// Stable caller identity. An opaque random ID is generated when omitted.
    #[arg(long)]
    device_id: Option<String>,
    /// Human-readable label; never used as identity.
    #[arg(long)]
    label: String,
    /// Local profile name. Defaults to the remote Device name or short ID.
    #[arg(long)]
    profile: Option<String>,
    /// Render non-secret connection metadata as JSON.
    #[arg(long)]
    json: bool,
  },
  /// Enable a paired device.
  // NOTICE(pairing-cli-compat): Keep the pair-ID form for existing scripts;
  // resource-oriented callers should use `auv devices enable <device>`.
  Enable { pair_id: String },
  /// Disable a paired device without deleting its history.
  // NOTICE(pairing-cli-compat): Keep the pair-ID form for existing scripts;
  // resource-oriented callers should use `auv devices disable <device>`.
  Disable { pair_id: String },
}

impl std::fmt::Debug for PairingCommand {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::CreateToken { ttl } => formatter.debug_struct("CreateToken").field("ttl", ttl).finish(),
      Self::Connect {
        device_id,
        label,
        profile,
        json,
        ..
      } => formatter
        .debug_struct("Connect")
        .field("token", &"[REDACTED]")
        .field("device_id", device_id)
        .field("label", label)
        .field("profile", profile)
        .field("json", json)
        .finish(),
      Self::Enable { pair_id } => formatter.debug_struct("Enable").field("pair_id", pair_id).finish(),
      Self::Disable { pair_id } => formatter.debug_struct("Disable").field("pair_id", pair_id).finish(),
    }
  }
}

#[cfg(test)]
#[path = "pairing_test.rs"]
mod tests;
