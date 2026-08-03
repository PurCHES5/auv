//! Foreground daemon process hosting and local endpoint publication.

mod daemon;
mod discovery;

pub(crate) use daemon::{default_local_listener, parse_listener, serve_foreground};
