#!/bin/bash
# Stop the Argus Docker stack.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE="docker compose -f $SCRIPT_DIR/docker-compose.yml"

# shellcheck source=scripts/argus-banner.sh
source "$SCRIPT_DIR/scripts/argus-banner.sh"
print_argus_banner

echo "  [*] Shutting down Argus stack..."
$COMPOSE down

echo ""
echo "  [+] Argus is down"
echo ""