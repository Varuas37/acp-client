//! Mock Agent Adapter
//!
//! A mock implementation of the Agent trait for testing.

use std::time::Duration;
use crate::domain::Agent;

/// Mock agent for testing purposes
#[derive(Debug, Clone)]
pub struct MockAgent {
    name: String,
    response: String,
}

impl MockAgent {
    /// Create a new mock agent
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            response: "Mock response".to_string(),
        }
    }

    /// Set a custom response
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response = response.into();
        self
    }

    /// Set a custom name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl Default for MockAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn cli_path(&self) -> &str {
        "echo"
    }

    fn acp_args(&self) -> Vec<String> {
        vec!["mock-acp".to_string()]
    }

    fn chat_args(&self) -> Vec<String> {
        vec![self.response.clone()]
    }

    fn requires_mcp_servers(&self) -> bool {
        false
    }

    fn session_init_delay(&self) -> Duration {
        Duration::ZERO
    }

    fn post_prompt_delay(&self) -> Duration {
        Duration::ZERO
    }

    fn process_response(&self, response: &str) -> String {
        response.to_string()
    }

    fn environment(&self) -> Vec<(String, String)> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_agent() {
        let agent = MockAgent::new();
        assert_eq!(agent.name(), "mock");
        assert_eq!(agent.cli_path(), "echo");
    }

    #[test]
    fn test_with_custom_response() {
        let agent = MockAgent::new().with_response("Custom response");
        let args = agent.chat_args();
        assert_eq!(args[0], "Custom response");
    }

    #[test]
    fn test_no_delays() {
        let agent = MockAgent::new();
        assert_eq!(agent.session_init_delay(), Duration::ZERO);
        assert_eq!(agent.post_prompt_delay(), Duration::ZERO);
    }
}
