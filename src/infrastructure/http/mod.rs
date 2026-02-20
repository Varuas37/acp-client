//! HTTP Server Infrastructure
//!
//! OpenAI-compatible HTTP API endpoints.

mod server;
mod types;

pub use server::{create_router, start_server, AppState};
pub use types::*;
