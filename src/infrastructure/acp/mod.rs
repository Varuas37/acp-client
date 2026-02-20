//! ACP Protocol Infrastructure
//!
//! Handles the low-level ACP (Agent Client Protocol) communication.

mod connection;
mod handler;
mod server_manager;

pub use connection::AcpConnection;
pub use handler::{AcpClientHandler, ResponseCollector};
pub use server_manager::{AcpServerManager, kiro as kiro_server};
