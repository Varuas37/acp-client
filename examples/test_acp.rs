//! Quick test to verify ACP client works

use acp_client::{AcpClient, Agent, AgentConfig, KiroAgent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("Testing ACP client...");

    // Create agent and config
    let agent = KiroAgent::new();
    println!("Agent: {}", agent.name());
    println!("CLI path: {}", agent.cli_path());

    let config = AgentConfig::new(agent.cli_path())
        .with_timeout(std::time::Duration::from_secs(30)); // 30 second timeout for test

    // Run in a blocking thread with its own runtime (same as Tauri does)
    let result = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        rt.block_on(async {
            let client = AcpClient::new(agent, config);
            println!("Sending test prompt: 'What is 2+2? Reply with just the number.'");
            client.send_prompt("What is 2+2? Reply with just the number.").await
        })
    }).await??;

    println!("Response: {}", result);
    println!("SUCCESS!");

    Ok(())
}
