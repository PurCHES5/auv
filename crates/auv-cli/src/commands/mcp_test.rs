use std::collections::BTreeMap;

use super::{McpInvokeInput, McpServer, core_invoke_adapters, mcp_command_inputs};

#[test]
fn default_mcp_server_accepts_its_invoke_registry_and_adapter_catalog() {
  McpServer::new(std::path::PathBuf::from(".")).expect("default MCP invoke catalogs should agree");
}

#[test]
fn mcp_disables_incidental_overlays_but_preserves_explicit_overlay_operations() {
  let incidental = mcp_command_inputs(auv_cli_invoke::InvokeNamespace::Window, pairs(&[("overlay", "true")]));
  assert_eq!(incidental.get("overlay").map(String::as_str), Some("false"));

  let explicit = mcp_command_inputs(auv_cli_invoke::InvokeNamespace::Overlay, BTreeMap::new());
  assert!(!explicit.contains_key("overlay"));
}

#[tokio::test]
async fn overlay_mcp_adapters_execute_the_shared_dry_run_commands() {
  let cases = [
    ("overlay.outline", pairs(&[("x", "10"), ("y", "20"), ("width", "120"), ("height", "40")])),
    ("overlay.cursor", pairs(&[("x", "10"), ("y", "20")])),
    ("overlay.status", pairs(&[("x", "10"), ("y", "20"), ("text", "processing")])),
    ("overlay.captureFrame", pairs(&[("x", "10"), ("y", "20"), ("width", "120"), ("height", "40")])),
    ("overlay.clickTarget", pairs(&[("x", "10"), ("y", "20"), ("width", "120"), ("height", "40")])),
  ];
  let adapters = core_invoke_adapters();

  for (command_id, inputs) in cases {
    let adapter = adapters.iter().find(|adapter| adapter.command_id == command_id).unwrap_or_else(|| panic!("missing {command_id} adapter"));
    adapter
      .invoke(McpInvokeInput {
        target_application_id: None,
        inputs,
        dry_run: true,
        cancellation: Default::default(),
      })
      .await
      .unwrap_or_else(|error| panic!("{command_id} MCP dry run failed: {error}"));
  }
}

// https://github.com/moeru-ai/auv/actions/runs/30577666189/job/90989876962
#[tokio::test]
async fn mcp_uses_the_same_typed_range_validation_as_cli() {
  // ROOT CAUSE:
  //
  // If invalid window-point coordinates were invoked outside macOS, the
  // platform rejection won because typed coordinate validation lived inside
  // the macOS-only command body.
  //
  // Before the fix, Linux CI observed a platform error instead of the shared
  // validation error. The fix validates command inputs before platform dispatch.
  let adapters = core_invoke_adapters();
  let adapter = adapters.iter().find(|adapter| adapter.command_id == "input.clickWindowPoint").expect("click-window-point adapter");
  let error = adapter
    .invoke(McpInvokeInput {
      target_application_id: None,
      inputs: pairs(&[("relative-x", "2"), ("relative-y", "0.5")]),
      dry_run: true,
      cancellation: Default::default(),
    })
    .await
    .expect_err("out-of-range MCP input must fail typed decoding");

  assert!(error.contains("within 0..=1"), "unexpected typed validation error: {error}");
}

fn pairs(values: &[(&str, &str)]) -> BTreeMap<String, String> {
  values.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect()
}
