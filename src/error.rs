//! Error types for the ACP client
//!
//! Generic error types that work with any agent implementation.

use thiserror::Error;

/// Error type for ACP operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to spawn agent CLI: {0}")]
    Spawn(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Create a spawn error
    pub fn spawn<S: Into<String>>(msg: S) -> Self {
        Error::Spawn(msg.into())
    }

    /// Create a connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Error::Connection(msg.into())
    }

    /// Create a session error
    pub fn session<S: Into<String>>(msg: S) -> Self {
        Error::Session(msg.into())
    }

    /// Create a protocol error
    pub fn protocol<S: Into<String>>(msg: S) -> Self {
        Error::Protocol(msg.into())
    }

    /// Create a session not found error
    pub fn session_not_found<S: Into<String>>(id: S) -> Self {
        Error::SessionNotFound(id.into())
    }

    /// Create an agent not found error
    pub fn agent_not_found<S: Into<String>>(name: S) -> Self {
        Error::AgentNotFound(name.into())
    }
}

/// Result type alias for ACP operations
pub type Result<T> = std::result::Result<T, Error>;

// Re-export as KiroError for backwards compatibility during transition
#[deprecated(since = "0.2.0", note = "Use Error instead")]
pub type KiroError = Error;
