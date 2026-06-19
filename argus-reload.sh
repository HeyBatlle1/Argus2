#!/bin/bash
# Reload vault secrets into running containers — no frontend rebuild.
# Use after:  argus vault set <key> <value>
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export ARGUS_BINARY="$SCRIPT_DIR/target/release/argus"
COMPOSE="docker compose -f $SCRIPT_DIR/docker-compose.yml"

# shellcheck source=scripts/argus-banner.sh
source "$SCRIPT_DIR/scripts/argus-banner.sh"
print_argus_banner

echo "  Secrets reload — inject fresh vault values into containers"
echo "  ─────────────────────────────────────────"

if [ ! -f "$ARGUS_BINARY" ]; then
  echo "  [!] Build argus first:  cargo build --release"
  exit 1
fi

# shellcheck source=scripts/vault-export.sh
source "$SCRIPT_DIR/scripts/vault-export.sh"
load_argus_secrets || exit 1

echo ""
echo "  [*] Recreating containers with fresh secrets..."
$COMPOSE up -d --force-recreate argus-daemon argus-workspace argus-frontend

echo ""
echo "  [+] Secrets reloaded — daemon, workspace, and frontend"
echo "  [*] No image rebuild required (WS token via /api/ws-token)"
echo ""