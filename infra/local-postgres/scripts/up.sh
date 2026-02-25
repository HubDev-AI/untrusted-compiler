#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$INFRA_DIR/.env"
ENV_EXAMPLE_FILE="$INFRA_DIR/.env.example"
COMPOSE_FILE="$INFRA_DIR/docker-compose.yml"

fail() {
  echo "error: $1" >&2
  exit 1
}

verify_postgres_credentials() {
  docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" exec -T postgres \
    env PGPASSWORD="$POSTGRES_PASSWORD" \
    psql \
      -h 127.0.0.1 \
      -p 5432 \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      -tAc "select 1" >/dev/null
}

if ! command -v docker >/dev/null 2>&1; then
  fail "docker is required"
fi

if ! docker compose version >/dev/null 2>&1; then
  fail "docker compose plugin is required"
fi

if [ ! -f "$COMPOSE_FILE" ]; then
  fail "missing compose file at $COMPOSE_FILE"
fi

if [ ! -f "$ENV_FILE" ]; then
  if [ ! -f "$ENV_EXAMPLE_FILE" ]; then
    fail "missing env example at $ENV_EXAMPLE_FILE"
  fi
  cp "$ENV_EXAMPLE_FILE" "$ENV_FILE"
  echo "created $ENV_FILE from .env.example"
fi

set -a
# shellcheck source=/dev/null
source "$ENV_FILE"
set +a

: "${POSTGRES_DB:?POSTGRES_DB must be set in $ENV_FILE}"
: "${POSTGRES_USER:?POSTGRES_USER must be set in $ENV_FILE}"
: "${POSTGRES_PASSWORD:?POSTGRES_PASSWORD must be set in $ENV_FILE}"

pushd "$INFRA_DIR" >/dev/null

docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --remove-orphans

container_id="$(docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" ps -q postgres)"
if [ -z "$container_id" ]; then
  popd >/dev/null
  fail "could not resolve postgres container id"
fi

for _ in $(seq 1 60); do
  health="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "$container_id" 2>/dev/null || true)"
  if [ "$health" = "healthy" ] || [ "$health" = "running" ]; then
    if ! verify_postgres_credentials; then
      echo "configured Postgres credentials failed verification (POSTGRES_USER=$POSTGRES_USER POSTGRES_DB=$POSTGRES_DB)" >&2
      echo "hint: existing local data may be initialized with different credentials; run $INFRA_DIR/scripts/reset.sh" >&2
      popd >/dev/null
      fail "postgres startup verification failed"
    fi
    echo "postgres is ready"
    echo "dsn: ${SEC4_RT_LASM_DB_POSTGRES_DSN:-postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@127.0.0.1:${PG_PORT:-5432}/$POSTGRES_DB?sslmode=disable}"
    echo "next: export SEC4_RT_LASM_DB_ADAPTER=postgres"
    echo "next: export SEC4_RT_LASM_DB_POSTGRES_DSN=..."
    popd >/dev/null
    exit 0
  fi
  if [ "$health" = "unhealthy" ]; then
    popd >/dev/null
    fail "postgres container became unhealthy"
  fi
  sleep 1
done

popd >/dev/null
fail "postgres did not become ready within timeout"
