#!/bin/bash
# Argus Docker launcher — pulls secrets from vault, never touches a plaintext file
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/argus"

if [ ! -f "$BINARY" ]; then
  echo "[!] Argus binary not found at $BINARY"
  echo "    Run: cargo build --release"
  exit 1
fi

echo "[*] Pulling secrets from vault..."
export OPENROUTER_API_KEY=$("$BINARY" vault get openrouter_api_key)
export TELEGRAM_BOT_TOKEN=$("$BINARY" vault get telegram_bot_token)

if [ -z "$OPENROUTER_API_KEY" ]; then
  echo "[!] openrouter_api_key not found in vault"
  exit 1
fi

echo "[+] Secrets loaded — starting Argus stack..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d "$@"
echo "[+] Done. Run 'docker compose logs -f' to watch."
