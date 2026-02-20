//! Google Gemini CLI Agent Adapter
//!
//! Implementation of the Agent trait for Google's Gemini CLI.
//!
//! Gemini CLI is an open-source AI agent that brings Gemini to your terminal.
//! Install: `npm install -g @anthropic/gemini-cli` or follow Google's instructions
//!
//! Usage modes:
//! - Interactive: `gemini`
//! - Non-interactive: `gemini -p "prompt"`
//! - With model: `gemini -m gemini-2.5-flash -p "prompt"`

use std::time::Duration;
use crate::domain::Agent;

/// Output format for Gemini CLI responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeminiOutputFormat {
    /// Plain text output (default)
    #[default]
    Text,
    /// JSON output
    Json,
    /// Streaming JSON (newline-delimited JSON events)
    StreamJson,
}

impl GeminiOutputFormat {
    fn as_str(&self) -> &'static str {
        match self {
            GeminiOutputFormat::Text => "text",
            GeminiOutputFormat::Json => "json",
            GeminiOutputFormat::StreamJson => "stream-json",
        }
    }
}

/// Google Gemini CLI agent implementation
///
/// Note: Gemini CLI may support MCP servers but doesn't appear to use ACP protocol.
/// This agent uses the non-interactive prompt mode (-p) for programmatic access.
#[derive(Debug, Clone)]
pub struct GeminiAgent {
    cli_path: String,
    model: Option<String>,
    output_format: GeminiOutputFormat,
    include_directories: Vec<String>,
}

impl GeminiAgent {
    /// Create a new Gemini agent with default settings
    pub fn new() -> Self {
        Self {
            cli_path: std::env::var("GEMINI_CLI_PATH")
                .unwrap_or_else(|_| "gemini".to_string()),
            model: std::env::var("GEMINI_MODEL").ok(),
            output_format: GeminiOutputFormat::default(),
            include_directories: vec![],
        }
    }

    /// Create a new Gemini agent with a custom CLI path
    pub fn with_cli_path(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            model: None,
            output_format: GeminiOutputFormat::default(),
            include_directories: vec![],
        }
    }

    /// Set the model to use (e.g., "gemini-2.5-flash", "gemini-2.5-pro")
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the output format
    pub fn with_output_format(mut self, format: GeminiOutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Add directories to include in context
    pub fn with_include_directories(mut self, dirs: Vec<String>) -> Self {
        self.include_directories = dirs;
        self
    }

    /// Add a single directory to include in context
    pub fn include_directory(mut self, dir: impl Into<String>) -> Self {
        self.include_directories.push(dir.into());
        self
    }
}

impl Default for GeminiAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Agent for GeminiAgent {
    fn name(&self) -> &str {
        "gemini"
    }

    fn cli_path(&self) -> &str {
        &self.cli_path
    }

    fn acp_args(&self) -> Vec<String> {
        // Gemini CLI doesn't support ACP, return chat args
        self.chat_args()
    }

    fn chat_args(&self) -> Vec<String> {
        let mut args = vec![
            "-p".to_string(), // Non-interactive prompt mode
        ];

        if let Some(ref model) = self.model {
            args.push("-m".to_string());
            args.push(model.clone());
        }

        // Add output format if not default text
        if self.output_format != GeminiOutputFormat::Text {
            args.push("--output-format".to_string());
            args.push(self.output_format.as_str().to_string());
        }

        // Add include directories
        if !self.include_directories.is_empty() {
            args.push("--include-directories".to_string());
            args.push(self.include_directories.join(","));
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
        // Gemini output is usually clean, but strip ANSI codes just in case
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
        let agent = GeminiAgent::new();
        assert_eq!(agent.name(), "gemini");
        let args = agent.chat_args();
        assert!(args.contains(&"-p".to_string()));
    }

    #[test]
    fn test_with_model() {
        let agent = GeminiAgent::new().with_model("gemini-2.5-flash");
        let args = agent.chat_args();
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"gemini-2.5-flash".to_string()));
    }

    #[test]
    fn test_json_output() {
        let agent = GeminiAgent::new().with_output_format(GeminiOutputFormat::Json);
        let args = agent.chat_args();
        assert!(args.contains(&"--output-format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_include_directories() {
        let agent = GeminiAgent::new()
            .include_directory("./src")
            .include_directory("./lib");
        let args = agent.chat_args();
        assert!(args.contains(&"--include-directories".to_string()));
        assert!(args.contains(&"./src,./lib".to_string()));
    }

    #[test]
    fn test_no_acp() {
        let agent = GeminiAgent::new();
        assert!(!agent.requires_mcp_servers());
    }
}
