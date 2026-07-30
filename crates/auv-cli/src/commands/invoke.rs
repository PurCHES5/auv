use clap::Args;

/// Invoke one core computer-use capability and record its run.
#[derive(Clone, Debug, Args)]
#[command(disable_help_flag = true)]
pub struct InvokeArgs {
  /// Command id and command-specific arguments, parsed by the invoke registry.
  // TODO(invoke-os-string-argv): non-UTF-8 invoke paths are deferred because
  // the current recorded InvokeRequest protocol stores inputs as String; reopen
  // when that owner-approved wire/storage contract can preserve OS path bytes.
  #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
  pub arguments: Vec<String>,
}
