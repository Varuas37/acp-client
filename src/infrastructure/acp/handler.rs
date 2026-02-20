//! ACP Client Handler
//!
//! Handles ACP protocol callbacks and response collection.

use std::sync::Arc;
use tokio::sync::Mutex;
use agent_client_protocol as acp;
use tracing::info;

/// Collects response text from ACP session notifications
pub struct ResponseCollector {
    text: Mutex<String>,
}

impl ResponseCollector {
    /// Create a new response collector
    pub fn new() -> Self {
        Self {
            text: Mutex::new(String::new()),
        }
    }

    /// Append text to the collected response
    pub async fn append(&self, s: &str) {
        let mut text = self.text.lock().await;
        text.push_str(s);
    }

    /// Get the collected response text
    pub async fn get(&self) -> String {
        self.text.lock().await.clone()
    }

    /// Clear the collected text
    pub async fn clear(&self) {
        self.text.lock().await.clear();
    }

    /// Check if any text has been collected
    pub async fn is_empty(&self) -> bool {
        self.text.lock().await.is_empty()
    }
}

impl Default for ResponseCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// ACP client handler that processes protocol callbacks
pub struct AcpClientHandler {
    collector: Arc<ResponseCollector>,
}

impl AcpClientHandler {
    /// Create a new handler with the given response collector
    pub fn new(collector: Arc<ResponseCollector>) -> Self {
        Self { collector }
    }

    /// Get the response collector
    pub fn collector(&self) -> &Arc<ResponseCollector> {
        &self.collector
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for AcpClientHandler {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        // Deny all permission requests for non-interactive use
        Ok(acp::RequestPermissionResponse::new(acp::RequestPermissionOutcome::Cancelled))
    }

    async fn write_text_file(
        &self,
        _args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: acp::CreateTerminalRequest,
    ) -> acp::Result<acp::CreateTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn kill_terminal_command(
        &self,
        _args: acp::KillTerminalCommandRequest,
    ) -> acp::Result<acp::KillTerminalCommandResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> std::result::Result<(), acp::Error> {
        info!("[ACP] session_notification received");

        match &args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                if let acp::ContentBlock::Text(text_content) = content {
                    info!("[ACP] Got text chunk: {} chars", text_content.text.len());
                    self.collector.append(&text_content.text).await;
                }
            }
            acp::SessionUpdate::AgentThoughtChunk(_) => {
                info!("[ACP] Got thought chunk (ignoring)");
            }
            _ => {
                info!("[ACP] Got other update type");
            }
        }
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_response_collector() {
        let collector = ResponseCollector::new();
        assert!(collector.is_empty().await);

        collector.append("Hello").await;
        collector.append(" World").await;

        assert_eq!(collector.get().await, "Hello World");
        assert!(!collector.is_empty().await);

        collector.clear().await;
        assert!(collector.is_empty().await);
    }
}
