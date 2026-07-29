//! Product MCP bootstrap for typed invoke adapters.
//!
//! Product-owned adapters call app domain APIs and map their values to MCP
//! presentation without executing the CLI registry.

use std::path::PathBuf;
use std::sync::Arc;

use auv_apple_textedit::DocumentWrite;
use auv_runtime::mcp::{McpInvokeAdapter, McpInvokeInput, McpInvokeSuccess};

/// Serve product MCP (CLI `auv mcp serve`) with product invoke metadata/adapters.
pub async fn serve_stdio(project_root: PathBuf) -> Result<(), String> {
  let registry = Arc::new(crate::product_registry());
  auv_runtime::mcp::serve_stdio_with_registry(project_root, registry, product_invoke_adapters()).await
}

/// Builds the product MCP server for embedded transports and tests.
pub fn server(project_root: PathBuf) -> Result<auv_runtime::mcp::McpServer, String> {
  auv_runtime::mcp::McpServer::with_registry(project_root, Arc::new(crate::product_registry()), product_invoke_adapters())
}

pub(crate) fn product_invoke_adapters() -> Vec<McpInvokeAdapter> {
  let mut adapters = auv_runtime::mcp::core_invoke_adapters();
  adapters.push(textedit_document_write_adapter());
  adapters
}

fn textedit_document_write_adapter() -> McpInvokeAdapter {
  McpInvokeAdapter::new(crate::integrations::textedit::DOCUMENT_WRITE_COMMAND_ID, |input| async move {
    invoke_textedit_document_write(input).await
  })
}

async fn invoke_textedit_document_write(input: McpInvokeInput) -> Result<McpInvokeSuccess, String> {
  invoke_textedit_document_write_with(input, |_| auv_apple_textedit::MacosTextEditDriver::open_local().map_err(|error| error.to_string()))
    .await
}

async fn invoke_textedit_document_write_with<D>(
  input: McpInvokeInput,
  open_driver: impl FnOnce(&DocumentWrite) -> Result<D, String>,
) -> Result<McpInvokeSuccess, String>
where
  D: auv_apple_textedit::TextEditDriver,
{
  reject_production_fixture_inputs(&input)?;
  let command = parse_document_write(&input)?;
  if input.dry_run {
    return Ok(McpInvokeSuccess::empty());
  }
  let driver = open_driver(&command)?;
  map_textedit_document_write(command, input.cancellation, driver).await.map(|(outcome, _)| outcome)
}

async fn map_textedit_document_write<D>(
  command: DocumentWrite,
  cancellation: auv_cli_invoke::InvokeCancellation,
  driver: D,
) -> Result<(McpInvokeSuccess, auv_apple_textedit::DocumentCommandReport), String>
where
  D: auv_apple_textedit::TextEditDriver,
{
  let report = crate::integrations::textedit::execute_document_write(command.clone(), cancellation, driver)
    .await
    .map_err(crate::integrations::textedit::DocumentWriteFailure::into_message)?;
  let outcome = document_write_outcome(&report)?;
  Ok((outcome, report))
}

fn document_write_outcome(report: &auv_apple_textedit::DocumentCommandReport) -> Result<McpInvokeSuccess, String> {
  McpInvokeSuccess::from_result(report)
}

fn reject_production_fixture_inputs(input: &McpInvokeInput) -> Result<(), String> {
  for name in ["driver", "fixture_observed_text"] {
    if input.inputs.contains_key(name) {
      return Err(format!("app.textedit.document.write does not accept --{name}"));
    }
  }
  Ok(())
}

fn parse_document_write(input: &McpInvokeInput) -> Result<DocumentWrite, String> {
  let content = input
    .inputs
    .get("content")
    .map(String::as_str)
    .ok_or_else(|| "app.textedit.document.write missing required flag --content".to_string())?;
  let mut command = DocumentWrite::defaults_with_content(content);
  if let Some(target) = &input.target_application_id {
    command.app_id = target.clone();
  }
  if let Some(replace) = input.inputs.get("replace") {
    command.replace = parse_bool(replace, "replace")?;
  }
  if let Some(verify) = input.inputs.get("verify") {
    command.verify = parse_bool(verify, "verify")?;
  }
  Ok(command)
}

fn parse_bool(value: &str, name: &str) -> Result<bool, String> {
  match value.trim().to_ascii_lowercase().as_str() {
    "true" | "1" | "yes" => Ok(true),
    "false" | "0" | "no" => Ok(false),
    other => Err(format!("invalid --{name} value {other}; expected true or false")),
  }
}
