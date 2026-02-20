//! ACP Server Manager
//!
//! Manages the lifecycle of the ACP server process (e.g., kiro-cli acp).
//! Ensures the server is running before client requests are made.

use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::domain::Agent;
use crate::error::{Error, Result};

/// Manages a single ACP server process
pub struct AcpServerManager {
    process: RwLock<Option<Child>>,
    cli_path: String,
    args: Vec<String>,
}

impl AcpServerManager {
    /// Create a new server manager for the given agent
    pub fn new<A: Agent>(agent: &A) -> Self {
        Self {
            process: RwLock::new(None),
            cli_path: agent.cli_path().to_string(),
            args: agent.acp_args(),
        }
    }

    /// Create a server manager with explicit CLI path and args
    pub fn with_config(cli_path: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            process: RwLock::new(None),
            cli_path: cli_path.into(),
            args,
        }
    }

    /// Check if the server process is running
    pub async fn is_running(&self) -> bool {
        let process = self.process.read().await;
        if let Some(ref child) = *process {
            // Check if process is still alive by trying to get its ID
            child.id().is_some()
        } else {
            false
        }
    }

    /// Start the ACP server if not already running
    pub async fn ensure_running(&self) -> Result<()> {
        // Check if already running
        {
            let process = self.process.read().await;
            if let Some(ref child) = *process {
                if child.id().is_some() {
                    info!("[ServerManager] ACP server already running (pid: {:?})", child.id());
                    return Ok(());
                }
            }
        }

        // Need to start the server
        self.start().await
    }

    /// Start the ACP server process
    pub async fn start(&self) -> Result<()> {
        let mut process = self.process.write().await;

        // Kill any existing process first
        if let Some(mut child) = process.take() {
            warn!("[ServerManager] Killing existing ACP server process");
            let _ = child.kill().await;
        }

        info!("[ServerManager] Starting ACP server: {} {:?}", self.cli_path, self.args);

        let mut cmd = Command::new(&self.cli_path);
        for arg in &self.args {
            cmd.arg(arg);
        }

        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(false) // Keep running even if manager is dropped
            .spawn()
            .map_err(|e| Error::spawn(format!("Failed to start ACP server: {}", e)))?;

        let pid = child.id();
        info!("[ServerManager] ACP server started (pid: {:?})", pid);

        *process = Some(child);

        // Give the server a moment to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(())
    }

    /// Stop the ACP server process
    pub async fn stop(&self) -> Result<()> {
        let mut process = self.process.write().await;

        if let Some(mut child) = process.take() {
            info!("[ServerManager] Stopping ACP server (pid: {:?})", child.id());
            child.kill().await
                .map_err(|e| Error::connection(format!("Failed to stop ACP server: {}", e)))?;
        }

        Ok(())
    }

    /// Health check - verify the server is running and responsive
    pub async fn health_check(&self) -> Result<bool> {
        if !self.is_running().await {
            return Ok(false);
        }

        // For now, just check if process is running
        // Could add actual protocol-level health check here
        Ok(true)
    }

    /// Restart the server
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.start().await
    }
}

impl Drop for AcpServerManager {
    fn drop(&mut self) {
        // Note: We don't kill the process on drop by default
        // because we want it to keep running for other requests
        // Use stop() explicitly if you want to terminate it
    }
}

/// Global server manager for Kiro
/// This ensures only one kiro-cli acp process runs at a time
pub mod kiro {
    use super::*;
    use once_cell::sync::Lazy;
    use tokio::sync::OnceCell;

    static KIRO_SERVER: Lazy<Arc<AcpServerManager>> = Lazy::new(|| {
        let cli_path = std::env::var("KIRO_CLI_PATH")
            .unwrap_or_else(|_| "kiro-cli".to_string());
        Arc::new(AcpServerManager::with_config(cli_path, vec!["acp".to_string()]))
    });

    /// Get the global Kiro server manager
    pub fn server() -> Arc<AcpServerManager> {
        KIRO_SERVER.clone()
    }

    /// Ensure the Kiro ACP server is running
    pub async fn ensure_running() -> Result<()> {
        server().ensure_running().await
    }

    /// Check if the Kiro ACP server is running
    pub async fn is_running() -> bool {
        server().is_running().await
    }

    /// Health check for the Kiro ACP server
    pub async fn health_check() -> Result<bool> {
        server().health_check().await
    }

    /// Stop the Kiro ACP server
    pub async fn stop() -> Result<()> {
        server().stop().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_manager_creation() {
        let manager = AcpServerManager::with_config("echo", vec!["test".to_string()]);
        assert!(!manager.is_running().await);
    }
}
