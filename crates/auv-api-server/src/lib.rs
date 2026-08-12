//! Protocol adapters for the AUV daemon control interface.
//!
//! Modules:
//! - `control`: transport-independent server contracts implemented by a daemon SDK.
//! - `server`: listener binding, request serving, and control routing.
//! - `runner_transport`: inherited private IPC for daemon-owned Runners.

mod authentication;
pub mod control;
mod middleware;
mod protocol;
mod rest;
pub mod runner_transport;
pub mod server;
