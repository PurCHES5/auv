use clap::{Args, Subcommand, ValueEnum};

#[derive(Clone, Debug, Args)]
pub struct RunnerArgs {
  #[command(subcommand)]
  pub command: RunnerCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum RunnerCommand {
  /// Create a daemon-owned Runner from a trusted RunnerClass.
  Create(CreateRunnerArgs),
  /// List daemon-owned Runners.
  #[command(visible_alias = "ls")]
  List(ListRunnersArgs),
  /// List trusted RunnerClasses registered by the selected Device.
  Classes(ListRunnersArgs),
  /// Get one Runner.
  Get(GetRunnerArgs),
  /// Stop and reap one Runner process.
  #[command(visible_alias = "delete")]
  Stop(StopRunnerArgs),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LifecycleArg {
  Ephemeral,
  UnlessIdle,
  UnlessShutdown,
}

#[derive(Clone, Debug, Args)]
pub struct CreateRunnerArgs {
  #[arg(long = "class")]
  pub runner_class: String,
  #[arg(long, value_enum, default_value = "unless-shutdown")]
  pub lifecycle: LifecycleArg,
  #[arg(long)]
  pub endpoint: Option<String>,
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct ListRunnersArgs {
  #[arg(long)]
  pub endpoint: Option<String>,
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct GetRunnerArgs {
  pub runner_id: String,
  #[arg(long)]
  pub endpoint: Option<String>,
  #[arg(long)]
  pub json: bool,
}

#[derive(Clone, Debug, Args)]
pub struct StopRunnerArgs {
  pub runner_id: String,
  /// Maximum seconds to wait before terminating a daemon-owned executable.
  /// Without this option, graceful draining has no deadline.
  #[arg(long, value_name = "SECONDS", conflicts_with = "force")]
  pub timeout: Option<u64>,
  /// Immediately terminate a daemon-owned executable. Remote endpoints are
  /// only detached and are never terminated by AUV.
  #[arg(long)]
  pub force: bool,
  #[arg(long)]
  pub endpoint: Option<String>,
  #[arg(long)]
  pub json: bool,
}
