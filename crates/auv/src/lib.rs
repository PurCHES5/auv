//! Domain-facing local and remote SDK facade for AUV.

pub mod client;
mod context;
pub mod discovery;
pub mod profile;

pub use client::Client;
pub use context::{AuvContext, ContextError};

pub(crate) fn resource_id_matches(canonical: &str, selector: &str) -> bool {
  fn compact(value: &str) -> String {
    ["device_", "run_", "runner_"]
      .iter()
      .find_map(|prefix| value.strip_prefix(prefix))
      .unwrap_or(value)
      .chars()
      .filter(|character| *character != '-')
      .collect()
  }

  canonical == selector || compact(canonical).starts_with(&compact(selector))
}
