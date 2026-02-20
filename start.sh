#!/bin/bash
#
# Kiro ACP Server - OpenAI-compatible API for kiro-cli
#
# This script sets up and runs the Kiro ACP server which provides
# OpenAI-compatible endpoints for kiro-cli.
#
# Usage:
#   ./start.sh                    # Start on default port 8080
#   PORT=3000 ./start.sh          # Start on custom port
#   KIRO_AGENT=my-agent ./start.sh # Use specific agent
#
# Environment variables:
#   PORT          - Server port (default: 8080)
#   KIRO_CLI_PATH - Path to kiro-cli binary (default: kiro-cli)
#   KIRO_AGENT    - Default agent to use (optional)
#   TIMEOUT_SECS  - Response timeout in seconds (default: 120)
#   RUST_LOG      - Log level (default: info)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "================================================"
echo "  Kiro ACP Server Setup"
echo "================================================"
echo ""

# =====================================================
# 1. Check Rust installation
# =====================================================
echo "[1/5] Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
RUST_VERSION=$(rustc --version)
echo "  ✓ $RUST_VERSION"

# =====================================================
# 2. Check Cargo
# =====================================================
echo "[2/5] Checking Cargo..."
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found even after Rust installation"
    exit 1
fi
CARGO_VERSION=$(cargo --version)
echo "  ✓ $CARGO_VERSION"

# =====================================================
# 3. Check kiro-cli
# =====================================================
echo "[3/5] Checking kiro-cli..."
export KIRO_CLI_PATH="${KIRO_CLI_PATH:-kiro-cli}"

if ! command -v "$KIRO_CLI_PATH" &> /dev/null; then
    # Try common installation paths
    if [ -f "$HOME/.toolbox/bin/kiro-cli" ]; then
        export KIRO_CLI_PATH="$HOME/.toolbox/bin/kiro-cli"
    elif [ -f "$HOME/.local/bin/kiro-cli" ]; then
        export KIRO_CLI_PATH="$HOME/.local/bin/kiro-cli"
    elif [ -f "/usr/local/bin/kiro-cli" ]; then
        export KIRO_CLI_PATH="/usr/local/bin/kiro-cli"
    else
        echo ""
        echo "Error: kiro-cli not found!"
        echo ""
        echo "Please install kiro-cli first. Options:"
        echo "  1. Install from https://kiro.dev"
        echo "  2. Or set KIRO_CLI_PATH to the full path of kiro-cli"
        echo ""
        echo "Example:"
        echo "  KIRO_CLI_PATH=/path/to/kiro-cli ./start.sh"
        exit 1
    fi
fi

KIRO_VERSION=$("$KIRO_CLI_PATH" --version 2>/dev/null || echo "unknown")
echo "  ✓ kiro-cli found at: $KIRO_CLI_PATH"
echo "  ✓ Version: $KIRO_VERSION"

# =====================================================
# 4. Build the server
# =====================================================
echo "[4/5] Building kiro-acp-server (release mode)..."
echo "  This may take a few minutes on first build..."
cargo build --release --bin kiro-acp-server 2>&1 | while read line; do
    # Show progress for Compiling lines
    if [[ "$line" == *"Compiling"* ]]; then
        echo "  $line"
    fi
done
echo "  ✓ Build complete"

# =====================================================
# 5. Start the server
# =====================================================
echo "[5/5] Starting server..."
echo ""

# Default environment
export PORT="${PORT:-8080}"
export TIMEOUT_SECS="${TIMEOUT_SECS:-120}"
export RUST_LOG="${RUST_LOG:-info}"

echo "================================================"
echo "  Server Configuration"
echo "================================================"
echo "  Port:        $PORT"
echo "  Kiro CLI:    $KIRO_CLI_PATH"
echo "  Timeout:     ${TIMEOUT_SECS}s"
echo "  Log level:   $RUST_LOG"
if [ -n "$KIRO_AGENT" ]; then
    echo "  Agent:       $KIRO_AGENT"
fi
echo "================================================"
echo ""

exec ./target/release/kiro-acp-server
