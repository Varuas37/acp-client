# kiro-acp-client

ACP (Agent Client Protocol) client for kiro-cli with OpenAI-compatible API.

## Quick Start

```bash
# Start the server
./start.sh

# Or with custom port
PORT=3000 ./start.sh
```

## OpenAI-Compatible API

Once running, you can use any OpenAI client library:

```bash
# Using curl
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "kiro",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

```python
# Using Python OpenAI client
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"  # kiro-cli handles auth
)

response = client.chat.completions.create(
    model="kiro",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)
```

## Endpoints

### OpenAI-Compatible
- `POST /v1/chat/completions` - Chat completion
- `GET /v1/models` - List available models

### Session Management
- `POST /v1/sessions` - Create a new session
- `GET /v1/sessions` - List all sessions
- `GET /v1/sessions/:id` - Get session details
- `DELETE /v1/sessions/:id` - Delete a session
- `POST /v1/sessions/:id/messages` - Send a message

### Health
- `GET /health` - Health check

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 8080 | Server port |
| `KIRO_CLI_PATH` | kiro-cli | Path to kiro-cli binary |
| `KIRO_AGENT` | (none) | Default agent to use |
| `TIMEOUT_SECS` | 120 | Response timeout |
| `RUST_LOG` | info | Log level |

## Library Usage

```rust
use kiro_acp_client::{KiroClient, KiroClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KiroClient::new(KiroClientConfig::default());

    // Create a session
    let session = client.create_session(Some("You are helpful".into())).await;

    // Chat
    let response = client.chat(&session.id, "Hello!").await?;
    println!("{}", response);

    Ok(())
}
```

## Architecture

This crate implements the [Agent Client Protocol (ACP)](https://agentclientprotocol.com)
to communicate with kiro-cli. It spawns `kiro-cli acp` as a subprocess and
communicates via JSON-RPC 2.0 over stdio.

The HTTP server wraps this client to provide OpenAI-compatible endpoints,
allowing any existing OpenAI client to work with kiro-cli.
