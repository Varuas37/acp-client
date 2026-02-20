//! ACP Connection Management
//!
//! Handles spawning agent CLI and managing ACP sessions.

use std::sync::Arc;
use tokio::process::Command;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use agent_client_protocol as acp;
use acp::Agent as _;
use tracing::info;

use crate::domain::{Agent, AgentConfig};
use crate::error::{Error, Result};
use super::handler::{AcpClientHandler, ResponseCollector};

/// ACP connection manager
pub struct AcpConnection;

impl AcpConnection {
    /// Run an ACP session with the given agent and prompt
    pub async fn run_session<A: Agent>(
        agent: &A,
        config: &AgentConfig,
        prompt: &str,
        collector: Arc<ResponseCollector>,
    ) -> Result<()> {
        info!("[ACP] Starting {} acp...", agent.name());

        // Build command
        let mut cmd = Command::new(agent.cli_path());
        for arg in agent.acp_args() {
            cmd.arg(arg);
        }

        // Add agent mode if specified
        if let Some(ref mode) = config.agent_mode {
            cmd.args(["--agent", mode]);
        }

        // Add extra args
        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        // Add environment variables
        for (key, value) in agent.environment() {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| Error::spawn(e.to_string()))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| Error::connection("Failed to get stdin"))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| Error::connection("Failed to get stdout"))?;

        let outgoing = stdin.compat_write();
        let incoming = stdout.compat();

        let handler = AcpClientHandler::new(collector);

        // Create ACP connection
        let (conn, handle_io) = acp::ClientSideConnection::new(
            handler,
            outgoing,
            incoming,
            |fut| {
                tokio::task::spawn_local(fut);
            },
        );

        // Handle I/O in the background
        tokio::task::spawn_local(handle_io);

        // Initialize
        info!("[ACP] Initializing...");
        let client_info = acp::Implementation::new("acp-client", env!("CARGO_PKG_VERSION"))
            .title("ACP Client");
        let init_request = acp::InitializeRequest::new(acp::ProtocolVersion::LATEST)
            .client_info(client_info);

        let init_response = conn.initialize(init_request)
            .await
            .map_err(|e| Error::protocol(format!("Initialize failed: {:?}", e)))?;

        info!("[ACP] Initialized: {:?}", init_response.agent_info);

        // Create session
        info!("[ACP] Creating session...");
        let cwd = config.working_dir
            .clone()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let session_request = acp::NewSessionRequest::new(cwd);

        let session_response = conn.new_session(session_request)
            .await
            .map_err(|e| Error::session(format!("Session creation failed: {:?}", e)))?;

        let session_id = session_response.session_id;
        info!("[ACP] Session created: {:?}", session_id);

        // Wait for agent to initialize (e.g., MCP servers)
        let init_delay = agent.session_init_delay();
        if !init_delay.is_zero() {
            tokio::time::sleep(init_delay).await;
        }

        // Send prompt with timeout
        info!("[ACP] Sending prompt ({} chars)...", prompt.len());
        let content = vec![acp::ContentBlock::Text(acp::TextContent::new(prompt.to_string()))];
        let prompt_request = acp::PromptRequest::new(session_id, content);

        let prompt_start = std::time::Instant::now();
        let prompt_response = tokio::time::timeout(
            config.timeout,
            conn.prompt(prompt_request)
        ).await
        .map_err(|_| Error::Timeout)?
        .map_err(|e| Error::protocol(format!("Prompt failed: {:?}", e)))?;

        info!("[ACP] Prompt completed in {:?}: {:?}", prompt_start.elapsed(), prompt_response.stop_reason);

        // Give time for final messages
        let post_delay = agent.post_prompt_delay();
        if !post_delay.is_zero() {
            tokio::time::sleep(post_delay).await;
        }

        info!("[ACP] Session completed");
        Ok(())
    }
}
