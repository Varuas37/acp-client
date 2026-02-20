//! ACP Protocol Infrastructure
//!
//! Handles the low-level ACP (Agent Client Protocol) communication.

mod connection;
mod handler;

pub use connection::AcpConnection;
pub use handler::{AcpClientHandler, ResponseCollector};
