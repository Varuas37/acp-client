//! OpenAI Codex CLI Agent Adapter
//!
//! Implementation of the Agent trait for OpenAI's Codex CLI.
//!
//! Codex CLI is a lightweight coding agent from OpenAI that runs in your terminal.
//! Install: `npm install -g @openai/codex` or `brew install --cask codex`
//!
//! Usage modes:
//! - Interactive: `codex`
//! - Non-interactive: `codex -q "prompt"` or `codex --quiet "prompt"`
//! - Full auto: `codex --approval-mode full-auto "prompt"`

use std::time::Duration;
use crate::domain::Agent;

/// Approval mode for Codex CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodexApprovalMode {
    /// Suggest mode - requires approval for all actions
    #[default]
    Suggest,
    /// Auto-edit mode - automatically approves file edits
    AutoEdit,
    /// Full-auto mode - automatically approves all actions (network disabled)
    FullAuto,
}

impl CodexApprovalMode {
    fn as_str(&self) -> &'static str {
        match self {
            CodexApprovalMode::Suggest => "suggest",
            CodexApprovalMode::AutoEdit => "auto-edit",
            CodexApprovalMode::FullAuto => "full-auto",
        }
    }
}

/// OpenAI Codex CLI agent implementation
///
/// Note: Codex CLI does not support ACP protocol, so this agent
/// uses the non-interactive quiet mode (-q) for programmatic access.
#[derive(Debug, Clone)]
pub struct CodexAgent {
    cli_path: String,
    model: Option<String>,
    approval_mode: CodexApprovalMode,
    json_output: bool,
}

impl CodexAgent {
    /// Create a new Codex agent with default settings
    pub fn new() -> Self {
        Self {
            cli_path: std::env::var("CODEX_CLI_PATH")
                .unwrap_or_else(|_| "codex".to_string()),
            model: std::env::var("CODEX_MODEL").ok(),
            approval_mode: CodexApprovalMode::default(),
            json_output: false,
        }
    }

    /// Create a new Codex agent with a custom CLI path
    pub fn with_cli_path(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            model: None,
            approval_mode: CodexApprovalMode::default(),
            json_output: false,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the approval mode
    pub fn with_approval_mode(mut self, mode: CodexApprovalMode) -> Self {
        self.approval_mode = mode;
        self
    }

    /// Enable JSON output format
    pub fn with_json_output(mut self, enabled: bool) -> Self {
        self.json_output = enabled;
        self
    }
}

impl Default for CodexAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Agent for CodexAgent {
    fn name(&self) -> &str {
        "codex"
    }

    fn cli_path(&self) -> &str {
        &self.cli_path
    }

    fn acp_args(&self) -> Vec<String> {
        // Codex does not support ACP, return chat args instead
        self.chat_args()
    }

    fn chat_args(&self) -> Vec<String> {
        let mut args = vec![
            "-q".to_string(), // Quiet/non-interactive mode
            "--approval-mode".to_string(),
            self.approval_mode.as_str().to_string(),
        ];

        if let Some(ref model) = self.model {
            args.push("-m".to_string());
            args.push(model.clone());
        }

        if self.json_output {
            args.push("--json".to_string());
        }

        args
    }

    fn requires_mcp_servers(&self) -> bool {
        false
    }

    fn session_init_delay(&self) -> Duration {
        Duration::ZERO
    }

    fn post_prompt_delay(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn process_response(&self, response: &str) -> String {
        // Strip ANSI codes and clean up output
        strip_ansi_codes(response)
    }

    fn environment(&self) -> Vec<(String, String)> {
        vec![
            // Suppress interactive UI elements
            ("CODEX_QUIET_MODE".to_string(), "1".to_string()),
        ]
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
        let agent = CodexAgent::new();
        assert_eq!(agent.name(), "codex");
        let args = agent.chat_args();
        assert!(args.contains(&"-q".to_string()));
        assert!(args.contains(&"--approval-mode".to_string()));
    }

    #[test]
    fn test_with_model() {
        let agent = CodexAgent::new().with_model("gpt-4");
        let args = agent.chat_args();
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"gpt-4".to_string()));
    }

    #[test]
    fn test_full_auto_mode() {
        let agent = CodexAgent::new().with_approval_mode(CodexApprovalMode::FullAuto);
        let args = agent.chat_args();
        assert!(args.contains(&"full-auto".to_string()));
    }

    #[test]
    fn test_json_output() {
        let agent = CodexAgent::new().with_json_output(true);
        let args = agent.chat_args();
        assert!(args.contains(&"--json".to_string()));
    }

    #[test]
    fn test_environment_vars() {
        let agent = CodexAgent::new();
        let env = agent.environment();
        assert!(env.iter().any(|(k, v)| k == "CODEX_QUIET_MODE" && v == "1"));
    }
}
