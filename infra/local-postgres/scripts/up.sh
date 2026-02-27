#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_ENV_FILE="$INFRA_DIR/.env"
ENV_FILE="${SEC4_LOCAL_POSTGRES_ENV_FILE:-$DEFAULT_ENV_FILE}"
ENV_EXAMPLE_FILE="$INFRA_DIR/.env.example"
COMPOSE_FILE="$INFRA_DIR/docker-compose.yml"
RUNTIME_ENV_FILE="$INFRA_DIR/.runtime.env"
AUTO_PORT="${SEC4_LOCAL_POSTGRES_AUTO_PORT:-1}"
AUTO_PORT_START="${SEC4_LOCAL_POSTGRES_AUTO_PORT_START:-55432}"
AUTO_PORT_END="${SEC4_LOCAL_POSTGRES_AUTO_PORT_END:-55599}"
LAST_STARTUP_FAILURE=""

fail() {
  echo "error: $1" >&2
  exit 1
}

compose_up() {
  docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --remove-orphans
}

compose_down_quiet() {
  docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" down >/dev/null 2>&1 || true
}

resolve_container_id() {
  docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" ps -q postgres
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

verify_postgres_host_route() {
  if ! command -v psql >/dev/null 2>&1; then
    return 0
  fi
  local current_user
  current_user="$(
    env PGPASSWORD="$POSTGRES_PASSWORD" \
      psql \
        -h 127.0.0.1 \
        -p "${PG_PORT:-5432}" \
        -U "$POSTGRES_USER" \
        -d "$POSTGRES_DB" \
        -v ON_ERROR_STOP=1 \
        -tAc "select current_user" 2>/dev/null || true
  )"
  current_user="$(printf '%s' "$current_user" | tr -d '[:space:]')"
  [ "$current_user" = "$POSTGRES_USER" ]
}

is_port_listening() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$port" -sTCP:LISTEN -n -P >/dev/null 2>&1
    return $?
  fi
  if command -v nc >/dev/null 2>&1; then
    nc -z 127.0.0.1 "$port" >/dev/null 2>&1
    return $?
  fi
  return 1
}

find_available_port() {
  local start="$1"
  local end="$2"
  local candidate
  for candidate in $(seq "$start" "$end"); do
    if ! is_port_listening "$candidate"; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

build_local_dsn_for_port() {
  local port="$1"
  printf 'postgres://%s:%s@127.0.0.1:%s/%s?sslmode=disable' \
    "$POSTGRES_USER" "$POSTGRES_PASSWORD" "$port" "$POSTGRES_DB"
}

redact_postgres_dsn_password() {
  local dsn="$1"
  printf '%s' "$dsn" | sed -E \
    -e 's#((postgres(ql)?://)[^:/?#]+:)[^@]*@#\1***@#g' \
    -e "s#([Pp][Aa][Ss][Ss][Ww][Oo][Rr][Dd][[:space:]]*=[[:space:]]*)'[^']*'#\\1'***'#g" \
    -e 's#([Pp][Aa][Ss][Ss][Ww][Oo][Rr][Dd][[:space:]]*=[[:space:]]*)[^[:space:]&]+#\1***#g'
}

write_runtime_env_file() {
  local dsn="$1"
  cat >"$RUNTIME_ENV_FILE" <<EOF
PG_PORT=$PG_PORT
POSTGRES_DB=$POSTGRES_DB
POSTGRES_USER=$POSTGRES_USER
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
SEC4_RT_LASM_DB_POSTGRES_DSN=$dsn
EOF
}

wait_for_startup_ready() {
  local container_id="$1"
  LAST_STARTUP_FAILURE=""
  for _ in $(seq 1 60); do
    local health
    health="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "$container_id" 2>/dev/null || true)"
    if [ "$health" = "healthy" ] || [ "$health" = "running" ]; then
      if ! verify_postgres_credentials; then
        LAST_STARTUP_FAILURE="credentials"
        return 1
      fi
      if ! verify_postgres_host_route; then
        LAST_STARTUP_FAILURE="host_route"
        return 2
      fi
      return 0
    fi
    if [ "$health" = "unhealthy" ]; then
      LAST_STARTUP_FAILURE="unhealthy"
      return 1
    fi
    sleep 1
  done
  LAST_STARTUP_FAILURE="timeout"
  return 1
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
  if [ -n "${SEC4_LOCAL_POSTGRES_ENV_FILE:-}" ]; then
    fail "configured env file does not exist: $ENV_FILE"
  fi
  if [ ! -f "$ENV_EXAMPLE_FILE" ]; then
    fail "missing env example at $ENV_EXAMPLE_FILE"
  fi
  cp "$ENV_EXAMPLE_FILE" "$DEFAULT_ENV_FILE"
  ENV_FILE="$DEFAULT_ENV_FILE"
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

rm -f "$RUNTIME_ENV_FILE"

compose_up
container_id="$(resolve_container_id)"
if [ -z "$container_id" ]; then
  popd >/dev/null
  fail "could not resolve postgres container id"
fi

if wait_for_startup_ready "$container_id"; then
  status=0
else
  status=$?
fi
if [ "$status" -ne 0 ]; then
  if [ "$status" -eq 2 ] && [ "$AUTO_PORT" = "1" ]; then
    replacement_port="$(find_available_port "$AUTO_PORT_START" "$AUTO_PORT_END" || true)"
    if [ -n "$replacement_port" ] && [ "$replacement_port" != "${PG_PORT:-5432}" ]; then
      echo "local postgres host route conflict detected on port ${PG_PORT:-5432}; retrying with free port $replacement_port"
      PG_PORT="$replacement_port"
      SEC4_RT_LASM_DB_POSTGRES_DSN="$(build_local_dsn_for_port "$replacement_port")"
      compose_down_quiet
      compose_up
      container_id="$(resolve_container_id)"
      if [ -z "$container_id" ]; then
        popd >/dev/null
        fail "could not resolve postgres container id after auto-port retry"
      fi
      if wait_for_startup_ready "$container_id"; then
        status=0
      else
        status=$?
      fi
    fi
  fi
  if [ "$status" -ne 0 ]; then
    case "$LAST_STARTUP_FAILURE" in
      credentials)
        echo "configured Postgres credentials failed verification (POSTGRES_USER=$POSTGRES_USER POSTGRES_DB=$POSTGRES_DB)" >&2
        echo "hint: existing local data may be initialized with different credentials; run $INFRA_DIR/scripts/reset.sh" >&2
        popd >/dev/null
        fail "postgres startup verification failed"
        ;;
      host_route)
        echo "configured host route verification failed for 127.0.0.1:${PG_PORT:-5432} (POSTGRES_USER=$POSTGRES_USER POSTGRES_DB=$POSTGRES_DB)" >&2
        echo "hint: another local Postgres service may shadow the docker-mapped port; set PG_PORT to an unused value (for example 55432) in $ENV_FILE and rerun $INFRA_DIR/scripts/reset.sh" >&2
        popd >/dev/null
        fail "postgres host route verification failed"
        ;;
      unhealthy)
        popd >/dev/null
        fail "postgres container became unhealthy"
        ;;
      *)
        popd >/dev/null
        fail "postgres did not become ready within timeout"
        ;;
    esac
  fi
fi

raw_dsn="${SEC4_RT_LASM_DB_POSTGRES_DSN:-$(build_local_dsn_for_port "${PG_PORT:-5432}")}"
write_runtime_env_file "$raw_dsn"
redacted_dsn="$(redact_postgres_dsn_password "$raw_dsn")"
echo "postgres is ready"
echo "dsn: $redacted_dsn"
echo "next: export SEC4_RT_LASM_DB_ADAPTER=postgres"
echo "next: export SEC4_RT_LASM_DB_POSTGRES_DSN=..."
popd >/dev/null
exit 0
