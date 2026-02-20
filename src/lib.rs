//! # acp-client
//!
//! A generic ACP (Agent Client Protocol) client library with OpenAI-compatible API.
//!
//! This crate provides:
//! - Generic ACP client that works with any agent (Kiro, Codex, Gemini, etc.)
//! - Session/conversation management
//! - OpenAI-compatible HTTP API server
//!
//! ## Supported Agents
//!
//! | Agent | CLI | ACP Support | Notes |
//! |-------|-----|-------------|-------|
//! | `KiroAgent` | kiro-cli | Yes | Full ACP protocol support |
//! | `CodexAgent` | codex | No | OpenAI Codex CLI, uses quiet mode |
//! | `GeminiAgent` | gemini | No | Google Gemini CLI, uses prompt mode |
//! | `MockAgent` | - | No | For testing only |
//!
//! ## Architecture
//!
//! This library follows Domain-Driven Design (DDD) principles:
//!
//! - **Domain Layer**: Core entities, value objects, and traits (`domain`)
//! - **Application Layer**: Use cases and orchestration (`application`)
//! - **Infrastructure Layer**: ACP protocol and HTTP server (`infrastructure`)
//! - **Adapters Layer**: Concrete agent implementations (`adapters`)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use acp_client::{AcpClient, AgentConfig, KiroAgent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = KiroAgent::new();
//!     let config = AgentConfig::new("kiro-cli");
//!     let client = AcpClient::new(agent, config);
//!
//!     let session = client.create_session(None).await;
//!     let response = client.chat(&session.id, "Hello!").await?;
//!
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Implementing Custom Agents
//!
//! To add support for a new CLI agent, implement the `Agent` trait:
//!
//! ```rust,no_run
//! use acp_client::domain::Agent;
//! use std::time::Duration;
//!
//! #[derive(Clone)]
//! struct MyAgent;
//!
//! #[async_trait::async_trait]
//! impl Agent for MyAgent {
//!     fn name(&self) -> &str { "my-agent" }
//!     fn cli_path(&self) -> &str { "my-cli" }
//!     fn acp_args(&self) -> Vec<String> { vec!["acp".into()] }
//!     fn chat_args(&self) -> Vec<String> { vec!["chat".into()] }
//! }
//! ```

// Domain Layer - Core business logic
pub mod domain;

// Application Layer - Use cases and orchestration
pub mod application;

// Infrastructure Layer - External integrations
pub mod infrastructure;

// Adapters Layer - Concrete implementations
pub mod adapters;

// Error types
pub mod error;

// Re-export commonly used types
pub use domain::{Agent, AgentConfig, AgentCapabilities, AgentInfo, Message, Session};
pub use domain::message::Role;
pub use application::{AcpClient, SessionService};
pub use adapters::{
    CodexAgent, CodexApprovalMode,
    GeminiAgent, GeminiOutputFormat,
    KiroAgent,
    MockAgent,
};
pub use error::{Error, Result};
pub use infrastructure::acp::{AcpConnection, ResponseCollector};
pub use infrastructure::http::{
    create_router, start_server, AppState,
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
    ErrorResponse, Model, ModelsResponse,
};

// Legacy exports for backwards compatibility (deprecated)
#[deprecated(since = "0.2.0", note = "Use AcpClient with KiroAgent instead")]
pub type KiroClient = AcpClient<KiroAgent>;

#[deprecated(since = "0.2.0", note = "Use AgentConfig instead")]
pub type KiroClientConfig = AgentConfig;
