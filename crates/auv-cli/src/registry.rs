//! Product invoke registry for the core command catalog.

use auv_cli_invoke::{InvokeRegistry, default_registry};

/// Product invoke registry used for CLI adapters and MCP catalog metadata.
pub fn product_registry() -> InvokeRegistry {
  default_registry()
}
