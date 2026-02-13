#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--dry-run] <impl> <endpoint:ping|decode|users-post> [base_url]" >&2
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

threads="${BENCH_THREADS:-$threads}"
conns="${BENCH_CONNECTIONS:-$conns}"
duration="${BENCH_DURATION:-$duration}"

declare -a cmd
case "$endpoint" in
  ping)
    target=10000
    target="${BENCH_TARGET_PING:-$target}"
    target="${BENCH_TARGET:-$target}"
    cmd=(wrk2 --latency -t"$threads" -c"$conns" -d"$duration" -R"$target" "${base_url}/ping")
    ;;
  decode)
    target=2000
    target="${BENCH_TARGET_DECODE:-$target}"
    target="${BENCH_TARGET:-$target}"
    cmd=(wrk2 --latency -t"$threads" -c"$conns" -d"$duration" -R"$target" -s "${root_dir}/load/wrk2/post_decode.lua" "$base_url")
    ;;
  users-post)
    target=500
    target="${BENCH_TARGET_USERS_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    cmd=(wrk2 --latency -t"$threads" -c"$conns" -d"$duration" -R"$target" -s "${root_dir}/load/wrk2/post_users.lua" "$base_url")
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

if [ "$dry_run" = "true" ]; then
  exit 0
fi

"${cmd[@]}" | tee "$raw"
"${root_dir}/scripts/wrk2_summary.sh" "$raw" "$impl" "$endpoint" "$target" "$summary"
