//! MCP bootstrap for core typed invoke adapters.

use std::path::PathBuf;
use std::sync::Arc;

/// Serve core MCP (CLI `auv mcp serve`) with core invoke metadata/adapters.
pub async fn serve_stdio(project_root: PathBuf) -> Result<(), String> {
  let registry = Arc::new(auv_cli_invoke::default_registry());
  auv_runtime::mcp::serve_stdio_with_registry(project_root, registry, auv_runtime::mcp::core_invoke_adapters()).await
}

/// Builds the core MCP server for embedded transports and tests.
pub fn server(project_root: PathBuf) -> Result<auv_runtime::mcp::McpServer, String> {
  auv_runtime::mcp::McpServer::with_registry(
    project_root,
    Arc::new(auv_cli_invoke::default_registry()),
    auv_runtime::mcp::core_invoke_adapters(),
  )
}
