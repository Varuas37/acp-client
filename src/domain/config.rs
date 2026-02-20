//! Agent configuration
//!
//! Configuration value objects for agents.

use std::time::Duration;

/// Configuration for an agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Path to the CLI executable
    pub cli_path: String,
    /// Specific agent/mode to use (e.g., "kiro_default", "amzn-builder")
    pub agent_mode: Option<String>,
    /// Model to use for completions
    pub model: Option<String>,
    /// Timeout for operations
    pub timeout: Duration,
    /// Extra CLI arguments
    pub extra_args: Vec<String>,
    /// Working directory for the agent
    pub working_dir: Option<String>,
}

impl AgentConfig {
    /// Create a new config with default values
    pub fn new(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            agent_mode: None,
            model: None,
            timeout: Duration::from_secs(120),
            extra_args: vec![],
            working_dir: None,
        }
    }

    /// Set the agent mode
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.agent_mode = Some(mode.into());
        self
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add extra CLI arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Set the working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new("acp-agent")
    }
}
