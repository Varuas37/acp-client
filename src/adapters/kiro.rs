//! Kiro Agent Adapter
//!
//! Implementation of the Agent trait for kiro-cli.

use std::time::Duration;
use crate::domain::Agent;

/// Kiro CLI agent implementation
#[derive(Debug, Clone)]
pub struct KiroAgent {
    cli_path: String,
    default_mode: Option<String>,
}

impl KiroAgent {
    /// Create a new Kiro agent with default settings
    pub fn new() -> Self {
        Self {
            cli_path: std::env::var("KIRO_CLI_PATH")
                .unwrap_or_else(|_| "kiro-cli".to_string()),
            default_mode: std::env::var("KIRO_AGENT").ok(),
        }
    }

    /// Create a new Kiro agent with a custom CLI path
    pub fn with_cli_path(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            default_mode: None,
        }
    }

    /// Set the default agent mode
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.default_mode = Some(mode.into());
        self
    }
}

impl Default for KiroAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Agent for KiroAgent {
    fn name(&self) -> &str {
        "kiro"
    }

    fn cli_path(&self) -> &str {
        &self.cli_path
    }

    fn acp_args(&self) -> Vec<String> {
        let mut args = vec!["acp".to_string()];
        if let Some(ref mode) = self.default_mode {
            args.push("--agent".to_string());
            args.push(mode.clone());
        }
        args
    }

    fn chat_args(&self) -> Vec<String> {
        let mut args = vec!["chat".to_string(), "--no-interactive".to_string()];
        if let Some(ref mode) = self.default_mode {
            args.push("--agent".to_string());
            args.push(mode.clone());
        }
        args
    }

    fn requires_mcp_servers(&self) -> bool {
        true
    }

    fn session_init_delay(&self) -> Duration {
        // Kiro needs time to initialize MCP servers
        Duration::from_secs(2)
    }

    fn post_prompt_delay(&self) -> Duration {
        // Small delay to ensure all messages are received
        Duration::from_millis(500)
    }

    fn process_response(&self, response: &str) -> String {
        // Strip ANSI escape codes from Kiro output
        strip_ansi_codes(response)
    }

    fn environment(&self) -> Vec<(String, String)> {
        vec![]
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07|\r").unwrap();
    re.replace_all(s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_agent() {
        let agent = KiroAgent::new();
        assert_eq!(agent.name(), "kiro");
        assert!(agent.acp_args().contains(&"acp".to_string()));
    }

    #[test]
    fn test_with_mode() {
        let agent = KiroAgent::new().with_mode("kiro_default");
        let args = agent.acp_args();
        assert!(args.contains(&"--agent".to_string()));
        assert!(args.contains(&"kiro_default".to_string()));
    }

    #[test]
    fn test_strip_ansi_codes() {
        let input = "\x1b[32mHello\x1b[0m World";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Hello World");
    }
}
