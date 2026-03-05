#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA_DIR="$INFRA_DIR/data"

"$SCRIPT_DIR/down.sh"

if [ -d "$DATA_DIR" ]; then
  rm -rf "$DATA_DIR"
  echo "removed $DATA_DIR"
fi

"$SCRIPT_DIR/up.sh"
