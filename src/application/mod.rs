//! Application Layer
//!
//! Use cases and application services that orchestrate domain logic.

mod client;
mod session_service;

pub use client::AcpClient;
pub use session_service::SessionService;
