//! ACP OpenAI-compatible Server
//!
//! Starts an HTTP server that exposes agents via OpenAI-compatible endpoints.

use acp_client::{Agent, AgentConfig, KiroAgent, start_server};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // Parse command line args
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let cli_path = env::var("KIRO_CLI_PATH")
        .unwrap_or_else(|_| "kiro-cli".to_string());

    let timeout: u64 = env::var("TIMEOUT_SECS")
        .unwrap_or_else(|_| "120".to_string())
        .parse()
        .expect("TIMEOUT_SECS must be a valid number");

    let agent_mode = env::var("KIRO_AGENT").ok();

    // Create the agent
    let agent = if let Some(ref mode) = agent_mode {
        KiroAgent::with_cli_path(&cli_path).with_mode(mode)
    } else {
        KiroAgent::with_cli_path(&cli_path)
    };

    // Create the config
    let config = AgentConfig::new(&cli_path)
        .with_timeout(Duration::from_secs(timeout));

    let config = if let Some(mode) = agent_mode.clone() {
        config.with_mode(mode)
    } else {
        config
    };

    tracing::info!("Starting ACP Server...");
    tracing::info!("Port: {}", port);
    tracing::info!("Agent: {}", agent.name());
    tracing::info!("CLI: {}", cli_path);
    tracing::info!("Timeout: {}s", timeout);
    if let Some(ref mode) = agent_mode {
        tracing::info!("Agent mode: {}", mode);
    }

    println!("\n🚀 ACP Server running at http://localhost:{}", port);
    println!("\nOpenAI-compatible endpoints:");
    println!("  POST /v1/chat/completions  - Chat completion");
    println!("  GET  /v1/models            - List models");
    println!("\nSession management:");
    println!("  POST /v1/sessions              - Create session");
    println!("  GET  /v1/sessions              - List sessions");
    println!("  GET  /v1/sessions/:id          - Get session");
    println!("  DELETE /v1/sessions/:id        - Delete session");
    println!("  POST /v1/sessions/:id/messages - Send message");
    println!("\nHealth check:");
    println!("  GET  /health");
    println!("\nExample usage with curl:");
    println!(r#"  curl http://localhost:{}/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{{"model": "default", "messages": [{{"role": "user", "content": "Hello!"}}]}}'"#, port);
    println!();

    start_server(agent, config, port).await?;

    Ok(())
}
