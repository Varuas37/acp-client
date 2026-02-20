//! Domain Layer
//!
//! Core business logic, entities, value objects, and traits.
//! This layer has no external dependencies.

mod agent;
mod config;
pub mod message;
mod session;

pub use agent::{Agent, AgentCapabilities, AgentInfo};
pub use config::AgentConfig;
pub use message::{Message, Role};
pub use session::Session;
