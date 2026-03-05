#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--dry-run] <impl> <endpoint:ping|decode|users-post|users-get|db-hot-write|db-hot-write-tx|db-hot-query-one|db-records> [base_url]" >&2
}

dry_run="false"
if [ "${1:-}" = "--dry-run" ]; then
  dry_run="true"
  shift
fi

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  usage
  exit 2
fi

impl="$1"
endpoint="$2"
base_url="${3:-http://127.0.0.1:8080}"

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
raw_dir="${root_dir}/results/raw"
sum_dir="${root_dir}/results/summaries"
mkdir -p "$raw_dir" "$sum_dir"

threads=4
conns=64
duration="60s"
target=""
raw="${raw_dir}/${impl}-${endpoint}.txt"
summary="${sum_dir}/${impl}-${endpoint}.json"
payload_path="${root_dir}/spec/payloads/user_4kb.json"
seed_user_id=""

threads="${BENCH_THREADS:-$threads}"
conns="${BENCH_CONNECTIONS:-$conns}"
duration="${BENCH_DURATION:-$duration}"

load_bin=""
load_supports_rate="false"

select_load_generator() {
  if command -v wrk2 >/dev/null 2>&1; then
    load_bin="wrk2"
    load_supports_rate="true"
    return
  fi
  if command -v wrk >/dev/null 2>&1; then
    load_bin="wrk"
    load_supports_rate="false"
    return
  fi
  echo "wrk2 or wrk is required but neither was found in PATH" >&2
  exit 127
}

select_load_generator

warn_wrk_fallback() {
  if [ "${load_supports_rate}" != "true" ]; then
    echo "warning: wrk2 not found; using wrk fallback without constant-rate -R enforcement" >&2
  fi
}

sample_rss_kb() {
  local pid="$1"
  local ps_rss=""
  if [ -z "$pid" ]; then
    echo ""
    return
  fi
  if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
    echo ""
    return
  fi
  if ! kill -0 "$pid" >/dev/null 2>&1; then
    echo ""
    return
  fi
  ps_rss="$(ps -o rss= -p "$pid" 2>/dev/null | awk 'NF { print $1; exit }')"
  if [[ "$ps_rss" =~ ^[0-9]+$ ]]; then
    echo "$ps_rss"
    return
  fi
  echo ""
}

build_wrk_cmd() {
  local script_path="${1:-}"
  local url="$2"
  local -a local_cmd

  local_cmd=("${load_bin}" --latency -t"${threads}" -c"${conns}" -d"${duration}")
  if [ "${load_supports_rate}" = "true" ]; then
    local_cmd+=(-R"${target}")
  fi
  if [ -n "${script_path}" ]; then
    local_cmd+=(-s "${script_path}")
  fi
  local_cmd+=("${url}")
  cmd=("${local_cmd[@]}")
}

declare -a cmd
case "$endpoint" in
  ping)
    target=10000
    target="${BENCH_TARGET_PING:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "" "${base_url}/ping"
    ;;
  decode)
    target=2000
    target="${BENCH_TARGET_DECODE:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "${root_dir}/load/wrk2/post_decode.lua" "${base_url}"
    ;;
  users-post)
    target=500
    target="${BENCH_TARGET_USERS_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "${root_dir}/load/wrk2/post_users.lua" "${base_url}"
    ;;
  users-get)
    target=2000
    target="${BENCH_TARGET_USERS_GET:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "${root_dir}/load/wrk2/get_user.lua" "${base_url}"
    ;;
  db-hot-write)
    target=1000
    target="${BENCH_TARGET_DB_HOT_WRITE:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "" "${base_url}/db/hot-write"
    ;;
  db-hot-write-tx)
    target=800
    target="${BENCH_TARGET_DB_HOT_WRITE_TX:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "" "${base_url}/db/hot-write-tx"
    ;;
  db-hot-query-one)
    target=700
    target="${BENCH_TARGET_DB_HOT_QUERY_ONE:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "" "${base_url}/db/hot-query-one"
    ;;
  db-records)
    target=1000
    target="${BENCH_TARGET_DB_RECORDS:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "" "${base_url}/db/records"
    ;;
  *)
    echo "unsupported endpoint: $endpoint" >&2
    usage
    exit 2
    ;;
esac

echo "profile impl=${impl} endpoint=${endpoint} targetRps=${target}"
echo "command: ${cmd[*]}"
echo "raw: $raw"
echo "summary: $summary"
warn_wrk_fallback
server_pid="${BENCH_SERVER_PID:-}"
if [ -n "$server_pid" ]; then
  echo "service pid: ${server_pid}"
fi

if [ "$dry_run" = "true" ]; then
  if [ "$endpoint" = "users-get" ] && [ -f "$payload_path" ]; then
    seed_user_id="$(jq -r '.id // empty' "$payload_path" 2>/dev/null || true)"
    if [ -n "$seed_user_id" ]; then
      echo "users-get seedUserId: $seed_user_id"
    fi
  fi
  exit 0
fi

{
  echo "# sec4-bench-load-bin=${load_bin}"
  echo "# sec4-bench-load-supports-rate=${load_supports_rate}"
} >"$raw"

if [ "$endpoint" = "users-get" ]; then
  if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required for users-get seed setup" >&2
    exit 127
  fi
  if ! command -v jq >/dev/null 2>&1; then
    echo "jq is required for users-get seed setup" >&2
    exit 127
  fi
  if [ ! -f "$payload_path" ]; then
    echo "seed payload not found: $payload_path" >&2
    exit 2
  fi
  seed_user_id="$(jq -r '.id // empty' "$payload_path")"
  if [ -z "$seed_user_id" ]; then
    echo "seed payload missing id field: $payload_path" >&2
    exit 2
  fi
  seed_status="$(curl -sS -o /dev/null -w "%{http_code}" -H "Content-Type: application/json" --data-binary "@${payload_path}" "${base_url}/users")"
  case "$seed_status" in
    200|201|409) ;;
    *)
      echo "failed to seed users-get benchmark user, status=${seed_status}" >&2
      exit 1
      ;;
  esac
  BENCH_USER_ID="$seed_user_id" "${cmd[@]}" | tee -a "$raw"
elif [ "$endpoint" = "decode" ] || [ "$endpoint" = "users-post" ]; then
  BENCH_PAYLOAD_FILE="$payload_path" "${cmd[@]}" | tee -a "$raw"
else
  "${cmd[@]}" | tee -a "$raw"
fi

rss_kb="$(sample_rss_kb "$server_pid")"
rss_source="unavailable"
if [ -n "$rss_kb" ]; then
  rss_source="ps"
fi

"${root_dir}/scripts/wrk2_summary.sh" "$raw" "$impl" "$endpoint" "$target" "$summary" "$rss_kb" "$rss_source"
