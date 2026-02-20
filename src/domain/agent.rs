//! Agent trait and related types
//!
//! Defines the abstract interface for AI agents that can be accessed via ACP.

use std::time::Duration;

/// Information about an agent
#[derive(Debug, Clone, Default)]
pub struct AgentInfo {
    /// Agent name
    pub name: String,
    /// Agent version
    pub version: Option<String>,
    /// Agent description
    pub description: Option<String>,
}

/// Capabilities advertised by an agent
#[derive(Debug, Clone, Default)]
pub struct AgentCapabilities {
    /// Whether the agent can load existing sessions
    pub load_session: bool,
    /// Whether the agent supports images in prompts
    pub image: bool,
    /// Whether the agent supports audio in prompts
    pub audio: bool,
    /// Available models
    pub available_models: Vec<String>,
    /// Available modes/agents
    pub available_modes: Vec<String>,
}

/// Trait defining the interface for an AI agent
///
/// Agents are the CLI tools that implement the ACP protocol.
/// Each agent type (Kiro, Claude Code, etc.) can have its own
/// implementation with specific quirks and behaviors.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's display name
    fn name(&self) -> &str;

    /// Get the path to the CLI executable
    fn cli_path(&self) -> &str;

    /// Get the CLI arguments for starting ACP mode
    fn acp_args(&self) -> Vec<String>;

    /// Get the CLI arguments for non-interactive chat mode (fallback)
    fn chat_args(&self) -> Vec<String>;

    /// Whether this agent requires mcpServers in session/new request
    fn requires_mcp_servers(&self) -> bool {
        true
    }

    /// Delay after session creation before sending prompts
    /// (some agents need time to initialize MCP servers)
    fn session_init_delay(&self) -> Duration {
        Duration::from_secs(2)
    }

    /// Delay after prompt completion before closing
    fn post_prompt_delay(&self) -> Duration {
        Duration::from_millis(500)
    }

    /// Process/clean response text (e.g., strip ANSI codes)
    fn process_response(&self, response: &str) -> String {
        response.to_string()
    }

    /// Get agent-specific environment variables
    fn environment(&self) -> Vec<(String, String)> {
        vec![]
    }
}
