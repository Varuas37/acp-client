//! Adapters Layer
//!
//! Concrete implementations of domain interfaces for specific agents.
//!
//! ## Available Agents
//!
//! | Agent | CLI | ACP Support | Notes |
//! |-------|-----|-------------|-------|
//! | KiroAgent | kiro-cli | Yes | Full ACP protocol support |
//! | CodexAgent | codex | No | Uses quiet mode (-q) |
//! | GeminiAgent | gemini | No | Uses prompt mode (-p) |
//! | MockAgent | - | No | For testing only |

mod codex;
mod gemini;
mod kiro;
mod mock;

pub use codex::{CodexAgent, CodexApprovalMode};
pub use gemini::{GeminiAgent, GeminiOutputFormat};
pub use kiro::KiroAgent;
pub use mock::MockAgent;
