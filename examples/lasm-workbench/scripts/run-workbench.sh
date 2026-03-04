#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PORT="${SEC4_LASM_WORKBENCH_PORT:-8080}"
DB_ADAPTER="${SEC4_LASM_WORKBENCH_DB_ADAPTER:-records}"
DB_BASE="${SEC4_LASM_WORKBENCH_DB_BASE:-$EXAMPLE_DIR/.lasm-workbench-db}"
WORKBENCH_ARGS=(
  --path
  "$EXAMPLE_DIR"
  --backend
  lasm
  --port
  "$PORT"
)

case "$DB_ADAPTER" in
  records|records.log)
    echo "LASM workbench: records adapter (persistence: records.log)"
    echo "DB base: $DB_BASE"
    WORKBENCH_ARGS+=(--db-base "$DB_BASE")
    ;;
  sqlite)
    echo "LASM workbench: sqlite adapter (persistence: records.sqlite3)"
    echo "DB base: $DB_BASE"
    WORKBENCH_ARGS+=(--db-base "$DB_BASE" --db-adapter sqlite)
    ;;
  postgres)
    echo "LASM workbench: postgres adapter"
    WORKBENCH_ARGS+=(--db-adapter postgres)
    if [[ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}" ]]; then
      WORKBENCH_ARGS+=(--db-postgres-dsn-file "$SEC4_RT_LASM_DB_POSTGRES_DSN_FILE")
    elif [[ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN:-}" ]]; then
      WORKBENCH_ARGS+=(--db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN")
    elif [[ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}" ]]; then
      WORKBENCH_ARGS+=(--db-postgres-dsn-file "$SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH")
    else
      echo "postgres adapter selected but no DSN provided" >&2
      echo "set one of SEC4_RT_LASM_DB_POSTGRES_DSN or "
      echo "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE to continue." >&2
      exit 1
    fi
    ;;
  *)
    echo "unsupported SEC4_LASM_WORKBENCH_DB_ADAPTER=$DB_ADAPTER" >&2
    echo "supported: records, sqlite, postgres" >&2
    exit 1
    ;;
esac

exec cargo run -p sec4 -- run "${WORKBENCH_ARGS[@]}"
