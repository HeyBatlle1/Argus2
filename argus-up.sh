#!/bin/bash
# Argus Docker launcher — pulls secrets from macOS Keychain or vault binary
# No plaintext .env file ever needed
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/argus"

echo "[*] Pulling secrets..."

# Strategy 1: Use vault binary if it exists (preferred)
if [ -f "$BINARY" ]; then
  export OPENROUTER_API_KEY=$("$BINARY" vault get openrouter_api_key 2>/dev/null)
  export TELEGRAM_BOT_TOKEN=$("$BINARY" vault get telegram_bot_token 2>/dev/null)
  echo "[+] Loaded from vault binary"

# Strategy 2: macOS Keychain fallback (when binary not built)
elif command -v security &>/dev/null; then
  echo "[!] Vault binary not found — trying macOS Keychain..."
  export OPENROUTER_API_KEY=$(security find-generic-password -a "argus" -s "openrouter_api_key" -w 2>/dev/null || echo "")
  export TELEGRAM_BOT_TOKEN=$(security find-generic-password -a "argus" -s "telegram_bot_token" -w 2>/dev/null || echo "")

  # If not in Keychain, prompt and save for next time
  if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "[!] OpenRouter key not in Keychain. Enter it now (will be saved):"
    read -rs OPENROUTER_API_KEY
    security add-generic-password -a "argus" -s "openrouter_api_key" -w "$OPENROUTER_API_KEY" 2>/dev/null || true
    echo "[+] Saved to Keychain"
  fi

  if [ -z "$TELEGRAM_BOT_TOKEN" ]; then
    echo "[!] Telegram token not in Keychain. Enter it now (will be saved):"
    read -rs TELEGRAM_BOT_TOKEN
    security add-generic-password -a "argus" -s "telegram_bot_token" -w "$TELEGRAM_BOT_TOKEN" 2>/dev/null || true
    echo "[+] Saved to Keychain"
  fi
  echo "[+] Loaded from macOS Keychain"

# Strategy 3: Nothing available
else
  echo "[!] No vault binary and no macOS Keychain available"
  echo "    Build the binary: cargo build --release"
  exit 1
fi

# Validate we have what we need
if [ -z "$OPENROUTER_API_KEY" ]; then
  echo "[!] OPENROUTER_API_KEY is empty — cannot start"
  exit 1
fi

if [ -z "$TELEGRAM_BOT_TOKEN" ]; then
  echo "[!] TELEGRAM_BOT_TOKEN is empty — Telegram will be disabled"
fi

echo "[+] Secrets loaded — starting Argus stack..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d "$@"
echo ""
echo "[+] Argus is up."
echo "    Frontend: http://localhost:3000"
echo "    WebSocket: ws://localhost:9000/ws"
echo "    Logs: docker compose logs -f"
