//! Infrastructure Layer
//!
//! External integrations and technical implementations.
//! This layer handles ACP protocol communication and HTTP server.

pub mod acp;
pub mod http;

pub use acp::{AcpConnection, AcpClientHandler, ResponseCollector};
