#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] <impl> <endpoint:wb-tasks-post|wb-tasks-with-comment|wb-task-comment-post|wb-task-get|wb-tasks-list> [base_url]

env:
  BENCH_REQUIRE_WRK2=1        Enforce wrk2-only load generation
  BENCH_WRK2_BIN=/abs/path    Explicit wrk2 binary path
USAGE
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

load_bin=""
load_supports_rate="false"
require_wrk2="${BENCH_REQUIRE_WRK2:-0}"
wrk2_bin_override="${BENCH_WRK2_BIN:-}"

is_truthy() {
  case "$1" in
    1|true|TRUE|yes|YES|on|ON)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

resolve_wrk2_bin() {
  if [ -n "$wrk2_bin_override" ] && [ -x "$wrk2_bin_override" ]; then
    printf '%s\n' "$wrk2_bin_override"
    return 0
  fi
  if command -v wrk2 >/dev/null 2>&1; then
    command -v wrk2
    return 0
  fi
  return 1
}

select_load_generator() {
  local wrk2_bin=""
  if wrk2_bin="$(resolve_wrk2_bin)"; then
    load_bin="$wrk2_bin"
    load_supports_rate="true"
    return
  fi
  if is_truthy "$require_wrk2"; then
    if [ -n "$wrk2_bin_override" ] && [ ! -x "$wrk2_bin_override" ]; then
      echo "wrk2 override path is not executable: ${wrk2_bin_override}" >&2
    fi
    echo "wrk2 is required for this run (set BENCH_REQUIRE_WRK2=0 to allow wrk fallback)" >&2
    exit 127
  fi
  if command -v wrk >/dev/null 2>&1; then
    load_bin="$(command -v wrk)"
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
declare -a endpoint_env

wb_task_id="${BENCH_WB_TASK_ID:-}"
wb_run_tag="${BENCH_WB_RUN_TAG:-$(date +%s%N)}"

case "$endpoint" in
  wb-tasks-post)
    target=500
    target="${BENCH_TARGET_WB_TASKS_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    endpoint_env=("BENCH_WB_RUN_TAG=${wb_run_tag}")
    build_wrk_cmd "${root_dir}/load/wrk2/post_wb_tasks.lua" "${base_url}"
    ;;
  wb-tasks-with-comment)
    target=350
    target="${BENCH_TARGET_WB_TASKS_WITH_COMMENT:-$target}"
    target="${BENCH_TARGET:-$target}"
    endpoint_env=("BENCH_WB_RUN_TAG=${wb_run_tag}")
    build_wrk_cmd "${root_dir}/load/wrk2/post_wb_tasks_with_comment.lua" "${base_url}"
    ;;
  wb-task-comment-post)
    target=500
    target="${BENCH_TARGET_WB_TASK_COMMENT_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    if [ -z "$wb_task_id" ] && [ "$dry_run" != "true" ]; then
      echo "BENCH_WB_TASK_ID is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    endpoint_env=("BENCH_WB_TASK_ID=${wb_task_id}" "BENCH_WB_RUN_TAG=${wb_run_tag}")
    build_wrk_cmd "${root_dir}/load/wrk2/post_wb_task_comment.lua" "${base_url}"
    ;;
  wb-task-get)
    target=2500
    target="${BENCH_TARGET_WB_TASK_GET:-$target}"
    target="${BENCH_TARGET:-$target}"
    if [ -z "$wb_task_id" ] && [ "$dry_run" != "true" ]; then
      echo "BENCH_WB_TASK_ID is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    endpoint_env=("BENCH_WB_TASK_ID=${wb_task_id}")
    build_wrk_cmd "${root_dir}/load/wrk2/get_wb_task.lua" "${base_url}"
    ;;
  wb-tasks-list)
    target=1500
    target="${BENCH_TARGET_WB_TASKS_LIST:-$target}"
    target="${BENCH_TARGET:-$target}"
    endpoint_env=()
    build_wrk_cmd "${root_dir}/load/wrk2/get_wb_tasks_list.lua" "${base_url}"
    ;;
  *)
    echo "unsupported workbench endpoint: $endpoint" >&2
    usage
    exit 2
    ;;
esac

echo "workbench profile impl=${impl} endpoint=${endpoint} targetRps=${target}"
echo "command: ${cmd[*]}"
echo "raw: $raw"
echo "summary: $summary"
warn_wrk_fallback
server_pid="${BENCH_SERVER_PID:-}"
if [ -n "$server_pid" ]; then
  echo "service pid: ${server_pid}"
fi

if [ "$dry_run" = "true" ]; then
  if [ -n "$wb_task_id" ]; then
    echo "workbench task id: $wb_task_id"
  fi
  if [ -n "$wb_run_tag" ]; then
    echo "workbench run tag: $wb_run_tag"
  fi
  exit 0
fi

{
  echo "# sec4-bench-load-bin=${load_bin}"
  echo "# sec4-bench-load-supports-rate=${load_supports_rate}"
} >"$raw"

if [ "${#endpoint_env[@]}" -gt 0 ]; then
  env "${endpoint_env[@]}" "${cmd[@]}" | tee -a "$raw"
else
  "${cmd[@]}" | tee -a "$raw"
fi

rss_kb="$(sample_rss_kb "$server_pid")"
rss_source="unavailable"
if [ -n "$rss_kb" ]; then
  rss_source="ps"
fi

"${root_dir}/scripts/wrk2_summary.sh" "$raw" "$impl" "$endpoint" "$target" "$summary" "$rss_kb" "$rss_source"
