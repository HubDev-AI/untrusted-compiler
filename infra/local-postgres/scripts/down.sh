#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$INFRA_DIR/.env"
COMPOSE_FILE="$INFRA_DIR/docker-compose.yml"

if ! command -v docker >/dev/null 2>&1; then
  echo "error: docker is required" >&2
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "error: docker compose plugin is required" >&2
  exit 1
fi

pushd "$INFRA_DIR" >/dev/null
if [ -f "$ENV_FILE" ]; then
  docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" down
else
  docker compose -f "$COMPOSE_FILE" down
fi
popd >/dev/null

echo "postgres is down"
