use std::path::PathBuf;

use clap::{Args, Subcommand};

/// Manage the lightweight AUV session API.
#[derive(Clone, Debug, Args)]
pub struct SessionArgs {
  #[command(subcommand)]
  pub command: SessionCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SessionCommand {
  /// Serve the lightweight AUV session API.
  #[command(
    after_long_help = "Examples:\n  # Serve on the default loopback address\n  auv session serve\n\n  # Use an explicit address and run store\n  auv session serve --host 127.0.0.1 --port 50051 --store-root .auv/runs"
  )]
  Serve(SessionServeArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SessionServeArgs {
  /// Host interface on which the session API listens.
  #[arg(long, default_value = crate::session_service::transport::DEFAULT_SESSION_API_HOST)]
  pub host: String,

  /// TCP port on which the session API listens.
  #[arg(long, default_value_t = crate::session_service::transport::DEFAULT_SESSION_API_PORT)]
  pub port: u16,

  /// Root directory used for recorded run data.
  #[arg(long, value_name = "PATH")]
  pub store_root: Option<PathBuf>,
}
