use clap::{Args, Subcommand};

/// Expose core AUV capabilities through MCP.
#[derive(Clone, Debug, Args)]
pub struct McpArgs {
  #[command(subcommand)]
  pub command: McpCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum McpCommand {
  /// Serve MCP over standard input and output.
  Serve,
}
