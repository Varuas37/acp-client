//! Session management service
//!
//! Application service for managing conversation sessions.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{Message, Session};
use crate::error::{Error, Result};

/// Service for managing sessions
#[derive(Debug, Clone)]
pub struct SessionService {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionService {
    /// Create a new session service
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create(&self, system_prompt: Option<String>) -> Session {
        let session = match system_prompt {
            Some(prompt) => Session::with_system_prompt(prompt),
            None => Session::new(),
        };

        let id = session.id.clone();
        self.sessions.write().await.insert(id, session.clone());
        session
    }

    /// Create a session with a title
    pub async fn create_with_title(
        &self,
        title: impl Into<String>,
        system_prompt: Option<String>,
    ) -> Session {
        let mut session = self.create(system_prompt).await;
        session.title = Some(title.into());
        let _ = self.update(session.clone()).await;
        session
    }

    /// Get a session by ID
    pub async fn get(&self, id: &str) -> Result<Session> {
        self.sessions
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| Error::session_not_found(id))
    }

    /// Update a session
    pub async fn update(&self, session: Session) -> Result<()> {
        let id = session.id.clone();
        if self.sessions.read().await.contains_key(&id) {
            self.sessions.write().await.insert(id, session);
            Ok(())
        } else {
            Err(Error::session_not_found(&id))
        }
    }

    /// Delete a session
    pub async fn delete(&self, id: &str) -> Result<Session> {
        self.sessions
            .write()
            .await
            .remove(id)
            .ok_or_else(|| Error::session_not_found(id))
    }

    /// List all sessions
    pub async fn list(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Add a message to a session
    pub async fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::session_not_found(session_id))?;
        session.add_message(message);
        Ok(())
    }

    /// Get or create a session
    pub async fn get_or_create(&self, id: &str) -> Session {
        if let Ok(session) = self.get(id).await {
            session
        } else {
            self.create(None).await
        }
    }

    /// Check if a session exists
    pub async fn exists(&self, id: &str) -> bool {
        self.sessions.read().await.contains_key(id)
    }

    /// Get session count
    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Clear all sessions
    pub async fn clear(&self) {
        self.sessions.write().await.clear();
    }
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let service = SessionService::new();
        let session = service.create(None).await;
        assert!(!session.id.is_empty());
    }

    #[tokio::test]
    async fn test_create_with_system_prompt() {
        let service = SessionService::new();
        let session = service.create(Some("Be helpful".into())).await;
        assert_eq!(session.system_prompt, Some("Be helpful".into()));
    }

    #[tokio::test]
    async fn test_get_session() {
        let service = SessionService::new();
        let session = service.create(None).await;
        let retrieved = service.get(&session.id).await.unwrap();
        assert_eq!(retrieved.id, session.id);
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let service = SessionService::new();
        let result = service.get("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let service = SessionService::new();
        service.create(None).await;
        service.create(None).await;
        let sessions = service.list().await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let service = SessionService::new();
        let session = service.create(None).await;
        let id = session.id.clone();
        service.delete(&id).await.unwrap();
        assert!(!service.exists(&id).await);
    }
}
