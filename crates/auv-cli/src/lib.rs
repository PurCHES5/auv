//! AUV core command frontend, MCP server, and session server.
//!
//! Supported crates own app/game behavior and command frontends. This crate
//! owns the root executable frontends over `auv-cli-invoke` and `auv-tracing`.

pub mod cli;
pub mod cli_frontend;
pub mod commands;
pub mod mcp;
pub mod plugin;
pub mod session_service;
pub mod xtask;
