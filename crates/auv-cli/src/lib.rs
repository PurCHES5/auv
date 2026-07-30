//! AUV core command frontend and MCP bootstrap.
//!
//! Supported crates own app/game behavior and command frontends. This crate
//! assembles only the root core CLI over `auv-runtime` and `auv-cli-invoke`.

pub mod cli;
pub mod cli_frontend;
pub mod commands;
pub mod mcp;
pub mod plugin;
pub mod xtask;
