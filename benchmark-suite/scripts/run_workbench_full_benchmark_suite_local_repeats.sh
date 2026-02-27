#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--keep-up] [--reset-db] [--infra-env path] [repeat-suite-args...]

Runs benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh
against repo-local Postgres infra at infra/local-postgres.

Local options:
  --dry-run     Print delegated benchmark plan; do not start/stop docker infra.
  --keep-up     Keep local Postgres infra running after suite completion.
  --reset-db    Recreate local Postgres data dir before running suite.
  --infra-env   Optional env file to load DSN defaults from (default: infra/.env,
                fallback: infra/.env.example in dry-run mode).

Any additional arguments are forwarded to run_workbench_full_benchmark_suite_repeats.sh.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
benchmark_root="$(cd "$script_dir/.." && pwd)"
repo_root="$(cd "$benchmark_root/.." && pwd)"
infra_root="$repo_root/infra/local-postgres"
infra_scripts_dir="$infra_root/scripts"
infra_up="$infra_scripts_dir/up.sh"
infra_down="$infra_scripts_dir/down.sh"
infra_reset="$infra_scripts_dir/reset.sh"
base_suite_script="$script_dir/run_workbench_full_benchmark_suite_repeats.sh"

if [ ! -x "$base_suite_script" ]; then
  echo "missing executable script: $base_suite_script" >&2
  exit 2
fi

if [ ! -x "$infra_up" ] || [ ! -x "$infra_down" ] || [ ! -x "$infra_reset" ]; then
  echo "missing local postgres infra scripts under: $infra_scripts_dir" >&2
  exit 2
fi

dry_run="false"
keep_up="false"
reset_db="false"
infra_env_override=""
suite_args=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    --keep-up)
      keep_up="true"
      shift
      ;;
    --reset-db)
      reset_db="true"
      shift
      ;;
    --infra-env)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      infra_env_override="$2"
      shift 2
      ;;
    --infra-env=*)
      infra_env_override="${1#--infra-env=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      while [ "$#" -gt 0 ]; do
        suite_args+=("$1")
        shift
      done
      ;;
    *)
      suite_args+=("$1")
      shift
      ;;
  esac
done

for arg in "${suite_args[@]}"; do
  case "$arg" in
    --lasm-db-adapter|--lasm-db-adapter=*|--lasm-postgres-dsn-file|--lasm-postgres-dsn-file=*)
      echo "conflicting suite argument in local wrapper: $arg" >&2
      echo "local wrapper manages --lasm-db-adapter=postgres and --lasm-postgres-dsn-file automatically" >&2
      exit 2
      ;;
  esac
done

resolve_infra_env_file() {
  if [ -n "$infra_env_override" ]; then
    printf '%s\n' "$infra_env_override"
    return
  fi
  if [ -f "$infra_root/.env" ]; then
    printf '%s\n' "$infra_root/.env"
    return
  fi
  printf '%s\n' "$infra_root/.env.example"
}

load_dsn_from_env_file() {
  local env_file="$1"
  if [ ! -f "$env_file" ]; then
    echo "local postgres env file not found: $env_file" >&2
    return 1
  fi
  set -a
  # shellcheck source=/dev/null
  source "$env_file"
  set +a
  local dsn="${SEC4_RT_LASM_DB_POSTGRES_DSN:-}"
  if [ -z "$dsn" ]; then
    local user="${POSTGRES_USER:-sec4}"
    local pass="${POSTGRES_PASSWORD:-sec4dev}"
    local port="${PG_PORT:-5432}"
    local db="${POSTGRES_DB:-sec4_local}"
    dsn="postgres://$user:$pass@127.0.0.1:$port/$db?sslmode=disable"
  fi
  dsn="$(printf '%s' "$dsn" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  if [ -z "$dsn" ]; then
    echo "resolved local postgres DSN is empty (env file: $env_file)" >&2
    return 1
  fi
  printf '%s\n' "$dsn"
}

started_infra="false"
needs_cleanup="false"
temp_dsn_file=""

cleanup() {
  local code=$?
  if [ -n "$temp_dsn_file" ] && [ -f "$temp_dsn_file" ]; then
    rm -f "$temp_dsn_file"
  fi
  if [ "$needs_cleanup" = "true" ] && [ "$keep_up" != "true" ] && [ "$dry_run" != "true" ]; then
    "$infra_down" >/dev/null 2>&1 || true
  fi
  exit "$code"
}
trap cleanup EXIT

if [ "$dry_run" = "true" ]; then
  echo "local postgres workbench full repeats mode: dry-run (no docker start/stop)"
else
  if [ "$reset_db" = "true" ]; then
    echo "local postgres workbench full repeats mode: reset + up"
    "$infra_reset"
  else
    echo "local postgres workbench full repeats mode: up"
    "$infra_up"
  fi
  started_infra="true"
  needs_cleanup="true"
fi

infra_env_file="$(resolve_infra_env_file)"
resolved_dsn="$(load_dsn_from_env_file "$infra_env_file")"

temp_dsn_file="$(mktemp)"
printf '%s\n' "$resolved_dsn" >"$temp_dsn_file"

run_cmd=(
  "$base_suite_script"
  --lasm-db-adapter postgres
  --lasm-postgres-dsn-file "$temp_dsn_file"
)
if [ "$dry_run" = "true" ]; then
  run_cmd+=(--dry-run)
fi
run_cmd+=("${suite_args[@]}")

echo "delegating: ${run_cmd[*]}"
"${run_cmd[@]}"

if [ "$dry_run" != "true" ] && [ "$keep_up" != "true" ] && [ "$started_infra" = "true" ]; then
  echo "local postgres workbench full repeats mode: down"
  "$infra_down"
  needs_cleanup="false"
fi

if [ "$dry_run" != "true" ] && [ "$keep_up" = "true" ]; then
  echo "local postgres workbench full repeats mode: kept running (--keep-up)"
fi
