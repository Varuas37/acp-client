//! Session entity
//!
//! Represents a conversation session with an agent.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::message::{Message, Role};

/// A conversation session with an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID (client-side)
    pub id: String,
    /// ACP session ID (from the agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acp_session_id: Option<String>,
    /// Session title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// System prompt for this session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Message history
    pub messages: Vec<Message>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub updated_at: DateTime<Utc>,
    /// Session metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            acp_session_id: None,
            title: None,
            system_prompt: None,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a session with a system prompt
    pub fn with_system_prompt(system_prompt: impl Into<String>) -> Self {
        let prompt = system_prompt.into();
        let mut session = Self::new();
        session.system_prompt = Some(prompt.clone());
        session.messages.push(Message::system(prompt));
        session
    }

    /// Create a session with a title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }

    /// Get the last N messages
    pub fn last_messages(&self, n: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if session has messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Build a prompt string from the message history
    pub fn build_prompt(&self) -> String {
        self.messages
            .iter()
            .map(|msg| {
                let prefix = match msg.role {
                    Role::System => "System",
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert!(!session.id.is_empty());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_with_system_prompt() {
        let session = Session::with_system_prompt("You are helpful");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::System);
        assert_eq!(session.messages[0].content, "You are helpful");
    }

    #[test]
    fn test_add_messages() {
        let mut session = Session::new();
        session.add_user_message("Hello");
        session.add_assistant_message("Hi there!");

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_build_prompt() {
        let mut session = Session::with_system_prompt("Be helpful");
        session.add_user_message("Hello");
        session.add_assistant_message("Hi!");

        let prompt = session.build_prompt();
        assert!(prompt.contains("System: Be helpful"));
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Assistant: Hi!"));
    }
}
