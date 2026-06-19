#!/bin/bash
# Argus startup — unlock vault once, inject secrets, start stack.
# Usage:  ./argus-up.sh          (normal start / restart)
#         ./argus-up.sh --build  (force-rebuild all images)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export ARGUS_BINARY="$SCRIPT_DIR/target/release/argus"
COMPOSE="docker compose -f $SCRIPT_DIR/docker-compose.yml"

# shellcheck source=scripts/argus-banner.sh
source "$SCRIPT_DIR/scripts/argus-banner.sh"
print_argus_banner

echo "  Startup sequence"
echo "  ─────────────────────────────────────────"

# ─── 1. Vault binary check ────────────────────────────────────────────────────

if [ ! -f "$ARGUS_BINARY" ]; then
  echo ""
  echo "  [!] Vault binary not found at $ARGUS_BINARY"
  echo "  [!] Build it first:  cargo build --release"
  exit 1
fi

# ─── 2. Load secrets from vault ───────────────────────────────────────────────

# shellcheck source=scripts/vault-export.sh
source "$SCRIPT_DIR/scripts/vault-export.sh"
load_argus_secrets || exit 1

# ─── 3. Start the stack ───────────────────────────────────────────────────────

FORCE_BUILD=0
for arg in "$@"; do
  [ "$arg" = "--build" ] && FORCE_BUILD=1 && break
done

echo ""
if [ "$FORCE_BUILD" = "1" ]; then
  echo "  [*] --build — rebuilding all images..."
  $COMPOSE up -d --build argus-daemon argus-workspace argus-frontend
else
  echo "  [*] Starting Argus stack..."
  $COMPOSE up -d argus-daemon argus-workspace argus-frontend
fi

echo ""
echo "  ─────────────────────────────────────────"
echo "  [+] Argus is up"
echo ""
echo "       Frontend  →  http://localhost:3000"
echo "       WebSocket →  ws://localhost:9000/ws"
echo "       Workspace →  http://localhost:8081"
echo ""
echo "       Reload keys:  ./argus-reload.sh"
echo "       Logs:         docker compose logs -f"
echo "       Stop:         ./argus-down.sh"
echo "  ─────────────────────────────────────────"
echo ""