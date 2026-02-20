//! ACP Client
//!
//! Main application service for interacting with agents via ACP.

use std::sync::Arc;
use tokio::task::LocalSet;
use tracing::{info, error, warn};

use crate::domain::{Agent, AgentConfig, Message, Session};
use crate::error::{Error, Result};
use crate::infrastructure::acp::{AcpConnection, ResponseCollector};
use super::SessionService;

/// Generic ACP client that works with any Agent implementation
pub struct AcpClient<A: Agent> {
    agent: A,
    config: AgentConfig,
    sessions: SessionService,
}

impl<A: Agent> AcpClient<A> {
    /// Create a new ACP client with the given agent and configuration
    pub fn new(agent: A, config: AgentConfig) -> Self {
        Self {
            agent,
            config,
            sessions: SessionService::new(),
        }
    }

    /// Get the agent
    pub fn agent(&self) -> &A {
        &self.agent
    }

    /// Get the configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get the session service
    pub fn sessions(&self) -> &SessionService {
        &self.sessions
    }

    /// Create a new session
    pub async fn create_session(&self, system_prompt: Option<String>) -> Session {
        self.sessions.create(system_prompt).await
    }

    /// Send a chat message in a session and get a response
    pub async fn chat(&self, session_id: &str, content: &str) -> Result<String> {
        // Get the session
        let mut session = self.sessions.get(session_id).await?;

        // Add user message
        session.add_user_message(content);

        // Build prompt from history
        let prompt = session.build_prompt();

        // Send and get response
        let response = self.send_prompt(&prompt).await?;

        // Add assistant response
        session.add_assistant_message(&response);

        // Update session
        self.sessions.update(session).await?;

        Ok(response)
    }

    /// Send a prompt and get a response
    pub async fn send_prompt(&self, prompt: &str) -> Result<String> {
        info!("[AcpClient] Sending prompt ({} chars) via {}", prompt.len(), self.agent.name());

        let agent = &self.agent;
        let config = self.config.clone();
        let prompt_owned = prompt.to_string();
        let prompt_for_fallback = prompt_owned.clone();

        let collector = Arc::new(ResponseCollector::new());
        let collector_clone = collector.clone();

        // Run everything in a LocalSet since the ACP Client trait is not Send
        let local = LocalSet::new();

        let result = local.run_until(async move {
            AcpConnection::run_session(agent, &config, &prompt_owned, collector_clone).await
        }).await;

        if let Err(e) = result {
            error!("[AcpClient] ACP session error: {}", e);
            return Err(e);
        }

        let response = collector.get().await;
        info!("[AcpClient] Response collected: {} chars", response.len());

        if response.is_empty() {
            // Fallback to non-interactive chat if ACP didn't return content
            warn!("[AcpClient] ACP returned empty, falling back to chat mode");
            return self.send_prompt_fallback(&prompt_for_fallback).await;
        }

        // Process response (e.g., strip ANSI codes)
        let processed = self.agent.process_response(&response);
        Ok(processed)
    }

    /// Fallback: Send a prompt via non-interactive chat
    async fn send_prompt_fallback(&self, prompt: &str) -> Result<String> {
        info!("[AcpClient] Using {} chat fallback", self.agent.name());

        use tokio::process::Command;
        use tokio::io::AsyncWriteExt;

        let mut cmd = Command::new(self.agent.cli_path());
        for arg in self.agent.chat_args() {
            cmd.arg(arg);
        }

        // Add agent environment
        for (key, value) in self.agent.environment() {
            cmd.env(key, value);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::spawn(e.to_string()))?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await
                .map_err(|e| Error::connection(e.to_string()))?;
            stdin.shutdown().await
                .map_err(|e| Error::connection(e.to_string()))?;
        }

        // Wait with timeout
        let output = tokio::time::timeout(
            self.config.timeout,
            child.wait_with_output()
        ).await
        .map_err(|_| Error::Timeout)?
        .map_err(|e| Error::connection(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let processed = self.agent.process_response(&stdout);

        if processed.trim().is_empty() {
            return Err(Error::protocol("Empty response from agent"));
        }

        Ok(processed)
    }

    /// Chat completion (OpenAI-compatible interface)
    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        _model: Option<&str>,
    ) -> Result<String> {
        // Build prompt from messages
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        self.send_prompt(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::MockAgent;

    #[tokio::test]
    async fn test_create_session() {
        let agent = MockAgent::new();
        let config = AgentConfig::new("mock-cli");
        let client = AcpClient::new(agent, config);

        let session = client.create_session(None).await;
        assert!(!session.id.is_empty());
    }

    #[tokio::test]
    async fn test_create_session_with_prompt() {
        let agent = MockAgent::new();
        let config = AgentConfig::new("mock-cli");
        let client = AcpClient::new(agent, config);

        let session = client.create_session(Some("Be helpful".into())).await;
        assert_eq!(session.system_prompt, Some("Be helpful".into()));
    }
}
