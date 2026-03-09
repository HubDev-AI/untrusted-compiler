#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] <impl> <endpoint:wb-tasks-post|wb-tasks-with-comment|wb-tasks-with-comment-tx|wb-task-comment-post|wb-task-get|wb-tasks-list> [base_url]

env:
  BENCH_REQUIRE_WRK2=1        Enforce wrk2-only load generation
  BENCH_WRK2_BIN=/abs/path    Explicit wrk2 binary path
  BENCH_WRK_FALLBACK_TIMEOUT  wrk fallback timeout (default: 10s)
  BENCH_SOCKET_ERROR_MAX_RATE_PCT
                              Max allowed socket error rate percentage before failure
                              (default: 0.50)
  BENCH_SOCKET_ERROR_MAX_RATE_PCT_<ENDPOINT>
                              Optional per-endpoint override where ENDPOINT is:
                              WB_TASKS_POST, WB_TASKS_WITH_COMMENT,
                              WB_TASKS_WITH_COMMENT_TX, WB_TASK_COMMENT_POST,
                              WB_TASK_GET, WB_TASKS_LIST
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
wrk_fallback_timeout_default="10s"
wrk_fallback_timeout="${BENCH_WRK_FALLBACK_TIMEOUT:-$wrk_fallback_timeout_default}"
socket_error_max_rate_pct_default="0.50"
socket_error_max_rate_pct_global="${BENCH_SOCKET_ERROR_MAX_RATE_PCT:-$socket_error_max_rate_pct_default}"
socket_error_max_rate_pct="$socket_error_max_rate_pct_global"

resolve_socket_error_threshold_endpoint_key() {
  case "$1" in
    wb-tasks-post) printf '%s\n' "WB_TASKS_POST" ;;
    wb-tasks-with-comment) printf '%s\n' "WB_TASKS_WITH_COMMENT" ;;
    wb-tasks-with-comment-tx) printf '%s\n' "WB_TASKS_WITH_COMMENT_TX" ;;
    wb-task-comment-post) printf '%s\n' "WB_TASK_COMMENT_POST" ;;
    wb-task-get) printf '%s\n' "WB_TASK_GET" ;;
    wb-tasks-list) printf '%s\n' "WB_TASKS_LIST" ;;
    *) printf '%s\n' "" ;;
  esac
}

resolve_socket_error_max_rate_pct() {
  local endpoint_key="$1"
  local endpoint_var=""
  local endpoint_override=""
  if [ -z "$endpoint_key" ]; then
    printf '%s\n' "$socket_error_max_rate_pct_global"
    return
  fi
  endpoint_var="BENCH_SOCKET_ERROR_MAX_RATE_PCT_${endpoint_key}"
  endpoint_override="${!endpoint_var:-}"
  if [ -n "$endpoint_override" ]; then
    printf '%s\n' "$endpoint_override"
  else
    printf '%s\n' "$socket_error_max_rate_pct_global"
  fi
}

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
  if [ -x "${root_dir}/bin/wrk2" ]; then
    printf '%s\n' "${root_dir}/bin/wrk2"
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
    echo "warning: wrk2 not found; using wrk fallback without constant-rate -R enforcement (timeout=${wrk_fallback_timeout})" >&2
  fi
}

require_template_renderer() {
  if command -v perl >/dev/null 2>&1; then
    return 0
  fi
  echo "perl is required to render benchmark wrk templates" >&2
  exit 127
}

require_template_renderer

sample_rss_kb() {
  local pid="$1"
  local fallback_pid=""
  local -a tree_pids=()
  local ps_rss=""
  local pid_rss=""
  local rss_sum=0
  local saw_rss="false"
  if [ -z "$pid" ]; then
    if [ -n "${BENCH_SERVER_PORT:-}" ]; then
      fallback_pid="$(resolve_listener_pid_by_port "${BENCH_SERVER_PORT}")"
      pid="$fallback_pid"
    fi
    if [ -z "$pid" ]; then
      echo ""
      return
    fi
  fi
  if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
    if [ -n "${BENCH_SERVER_PORT:-}" ]; then
      fallback_pid="$(resolve_listener_pid_by_port "${BENCH_SERVER_PORT}")"
      if [[ "$fallback_pid" =~ ^[0-9]+$ ]]; then
        pid="$fallback_pid"
      else
        echo ""
        return
      fi
    else
      echo ""
      return
    fi
  fi
  collect_process_tree_pids "$pid" tree_pids
  if [ "${#tree_pids[@]}" -eq 0 ]; then
    if [ -n "${BENCH_SERVER_PORT:-}" ]; then
      fallback_pid="$(resolve_listener_pid_by_port "${BENCH_SERVER_PORT}")"
      if [[ "$fallback_pid" =~ ^[0-9]+$ ]] && [ "$fallback_pid" != "$pid" ]; then
        collect_process_tree_pids "$fallback_pid" tree_pids
      fi
    fi
    if [ "${#tree_pids[@]}" -eq 0 ]; then
      echo ""
      return
    fi
  fi
  for pid_rss in "${tree_pids[@]}"; do
    ps_rss="$(ps -o rss= -p "$pid_rss" 2>/dev/null | awk 'NF { print $1; exit }')"
    if [[ "$ps_rss" =~ ^[0-9]+$ ]]; then
      rss_sum=$((rss_sum + ps_rss))
      saw_rss="true"
    fi
  done
  if [ "$saw_rss" = "true" ]; then
    echo "$rss_sum"
    return
  fi
  echo ""
}

resolve_listener_pid_by_port() {
  local port="$1"
  local pid=""
  if [ -z "$port" ] || ! [[ "$port" =~ ^[0-9]+$ ]]; then
    return 1
  fi
  if command -v lsof >/dev/null 2>&1; then
    pid="$(lsof -nP -t -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NF { print; exit }')"
    if [[ "$pid" =~ ^[0-9]+$ ]]; then
      printf '%s\n' "$pid"
      return 0
    fi
  fi
  if command -v ss >/dev/null 2>&1; then
    pid="$(ss -ltnp "sport = :$port" 2>/dev/null | awk -F'pid=|,' '/pid=/{print $2; exit}')"
    if [[ "$pid" =~ ^[0-9]+$ ]]; then
      printf '%s\n' "$pid"
      return 0
    fi
  fi
  return 1
}

collect_process_tree_pids() {
  local root_pid="$1"
  local out_var="$2"
  local current_pid=""
  local child_pid=""
  local children=""
  local -a queue=()
  local -a seen=()
  local -a collected=()

  if [ -z "$root_pid" ] || ! [[ "$root_pid" =~ ^[0-9]+$ ]]; then
    eval "$out_var=()"
    return
  fi

  queue=("$root_pid")
  while [ "${#queue[@]}" -gt 0 ]; do
    current_pid="${queue[0]}"
    queue=("${queue[@]:1}")

    case " ${seen[*]} " in
      *" ${current_pid} "*) continue ;;
    esac
    seen+=("$current_pid")

    if kill -0 "$current_pid" >/dev/null 2>&1; then
      collected+=("$current_pid")
    fi

    children="$(pgrep -P "$current_pid" 2>/dev/null || true)"
    if [ -z "$children" ]; then
      continue
    fi
    while IFS= read -r child_pid; do
      if [ -n "$child_pid" ]; then
        queue+=("$child_pid")
      fi
    done <<EOF
$children
EOF
  done

  eval "$out_var=(\"\${collected[@]}\")"
}

tmp_wrk_scripts=()
cleanup_tmp_wrk_scripts() {
  local script_path=""
  for script_path in "${tmp_wrk_scripts[@]}"; do
    [ -f "$script_path" ] && rm -f "$script_path"
  done
}
trap cleanup_tmp_wrk_scripts EXIT

prepare_wrk_script() {
  local template_path="$1"
  shift
  if [ "$#" -eq 0 ]; then
    printf '%s\n' "$template_path"
    return 0
  fi

  local rendered_path
  local template_stem
  mkdir -p "${root_dir}/results/tmp-wrk"
  template_stem="$(basename "$template_path")"
  template_stem="${template_stem%.lua}"
  rendered_path="$(mktemp "${root_dir}/results/tmp-wrk/${template_stem}.XXXXXX")"
  cp "$template_path" "$rendered_path"
  local replacement=""
  for replacement in "$@"; do
    local placeholder="${replacement%%=*}"
    local value="${replacement#*=}"
    PLACEHOLDER="$placeholder" REPLACEMENT="$value" perl -0pi -e 's/\Q$ENV{PLACEHOLDER}\E/$ENV{REPLACEMENT}/g' "$rendered_path"
  done
  tmp_wrk_scripts+=("$rendered_path")
  printf '%s\n' "$rendered_path"
}

build_wrk_cmd() {
  local script_path="${1:-}"
  local url="$2"
  local -a local_cmd

  local_cmd=("${load_bin}" --latency -t"${threads}" -c"${conns}" -d"${duration}")
  if [ "${load_supports_rate}" = "true" ]; then
    local_cmd+=(-R"${target}")
  else
    local_cmd+=(--timeout "${wrk_fallback_timeout}")
  fi
  if [ -n "${script_path}" ]; then
    local_cmd+=(-s "${script_path}")
  fi
  local_cmd+=("${url}")
  cmd=("${local_cmd[@]}")
}

declare -a cmd

wb_task_id="${BENCH_WB_TASK_ID:-}"
wb_run_tag="${BENCH_WB_RUN_TAG:-$(date +%s%N)}"

case "$endpoint" in
  wb-tasks-post)
    target=500
    target="${BENCH_TARGET_WB_TASKS_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd \
      "$(prepare_wrk_script "${root_dir}/load/wrk2/post_wb_tasks.lua" "__BENCH_WB_RUN_TAG__=${wb_run_tag}")" \
      "${base_url}"
    ;;
  wb-tasks-with-comment)
    target=200
    target="${BENCH_TARGET_WB_TASKS_WITH_COMMENT:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd \
      "$(prepare_wrk_script "${root_dir}/load/wrk2/post_wb_tasks_with_comment.lua" "__BENCH_WB_RUN_TAG__=${wb_run_tag}")" \
      "${base_url}"
    ;;
  wb-tasks-with-comment-tx)
    target=200
    target="${BENCH_TARGET_WB_TASKS_WITH_COMMENT_TX:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd \
      "$(prepare_wrk_script "${root_dir}/load/wrk2/post_wb_tasks_with_comment_tx.lua" "__BENCH_WB_RUN_TAG__=${wb_run_tag}")" \
      "${base_url}"
    ;;
  wb-task-comment-post)
    target=500
    target="${BENCH_TARGET_WB_TASK_COMMENT_POST:-$target}"
    target="${BENCH_TARGET:-$target}"
    if [ -z "$wb_task_id" ] && [ "$dry_run" != "true" ]; then
      echo "BENCH_WB_TASK_ID is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    build_wrk_cmd \
      "$(prepare_wrk_script "${root_dir}/load/wrk2/post_wb_task_comment.lua" "__BENCH_WB_TASK_ID__=${wb_task_id}" "__BENCH_WB_RUN_TAG__=${wb_run_tag}")" \
      "${base_url}"
    ;;
  wb-task-get)
    target=2500
    target="${BENCH_TARGET_WB_TASK_GET:-$target}"
    target="${BENCH_TARGET:-$target}"
    if [ -z "$wb_task_id" ] && [ "$dry_run" != "true" ]; then
      echo "BENCH_WB_TASK_ID is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    build_wrk_cmd \
      "$(prepare_wrk_script "${root_dir}/load/wrk2/get_wb_task.lua" "__BENCH_WB_TASK_ID__=${wb_task_id}")" \
      "${base_url}"
    ;;
  wb-tasks-list)
    target=1500
    target="${BENCH_TARGET_WB_TASKS_LIST:-$target}"
    target="${BENCH_TARGET:-$target}"
    build_wrk_cmd "${root_dir}/load/wrk2/get_wb_tasks_list.lua" "${base_url}"
    ;;
  *)
    echo "unsupported workbench endpoint: $endpoint" >&2
    usage
    exit 2
    ;;
esac

socket_error_threshold_endpoint_key="$(resolve_socket_error_threshold_endpoint_key "$endpoint")"
socket_error_max_rate_pct="$(resolve_socket_error_max_rate_pct "$socket_error_threshold_endpoint_key")"

echo "workbench profile impl=${impl} endpoint=${endpoint} targetRps=${target}"
echo "socketErrorMaxRatePct: ${socket_error_max_rate_pct}"
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

write_raw_header() {
  {
    echo "# sec4-bench-load-bin=${load_bin}"
    echo "# sec4-bench-load-supports-rate=${load_supports_rate}"
    echo "# sec4-bench-wrk-fallback-timeout=${wrk_fallback_timeout}"
  } >"$raw"
}

run_profile_command_with_retry() {
  local max_attempts=1
  local attempt=1
  local exit_code=0
  local wrk2_assertion_pattern='response_complete: Assertion'

  if [ "${load_supports_rate}" = "true" ]; then
    max_attempts=2
  fi

  while [ "$attempt" -le "$max_attempts" ]; do
    if [ "$attempt" -gt 1 ]; then
      write_raw_header
      echo "# sec4-bench-retry-attempt=${attempt}" >>"$raw"
      echo "warning: wrk2 assertion failure detected; retrying once (impl=${impl}, endpoint=${endpoint})" >&2
    fi

    if "${cmd[@]}" 2>&1 | tee -a "$raw"; then
      return 0
    fi
    exit_code=$?

    if [ "${load_supports_rate}" != "true" ]; then
      return "$exit_code"
    fi
    if ! grep -q "$wrk2_assertion_pattern" "$raw"; then
      return "$exit_code"
    fi
    if [ "$attempt" -ge "$max_attempts" ]; then
      return "$exit_code"
    fi

    attempt=$((attempt + 1))
  done

  return "$exit_code"
}

write_raw_header

if ! run_profile_command_with_retry; then
  exit 1
fi

rss_kb="$(sample_rss_kb "$server_pid")"
rss_source="unavailable"
if [ -n "$rss_kb" ]; then
  rss_source="ps"
fi

"${root_dir}/scripts/wrk2_summary.sh" "$raw" "$impl" "$endpoint" "$target" "$summary" "$rss_kb" "$rss_source"

non_2xx_or_3xx="$(jq -r '.http.non2xxOr3xxResponses // 0' "$summary" 2>/dev/null || echo 0)"
if [ "${non_2xx_or_3xx}" != "0" ]; then
  echo "workbench profile failed: impl=${impl} endpoint=${endpoint} observed non-2xx/3xx responses=${non_2xx_or_3xx}" >&2
  exit 1
fi

completed_requests="$(jq -r '.completedRequests // 0' "$summary" 2>/dev/null || echo 0)"
if [ "${completed_requests}" = "0" ]; then
  echo "workbench profile failed: impl=${impl} endpoint=${endpoint} completedRequests=0" >&2
  exit 1
fi

socket_error_total="$(
  jq -r '(.http.socketErrors.connect // 0) + (.http.socketErrors.read // 0) + (.http.socketErrors.write // 0) + (.http.socketErrors.timeout // 0)' \
    "$summary" 2>/dev/null || echo 0
)"
if [ "${socket_error_total}" != "0" ]; then
  total_with_socket_errors="$((completed_requests + socket_error_total))"
  if [ "$total_with_socket_errors" -le 0 ]; then
    total_with_socket_errors="$socket_error_total"
  fi
  socket_error_rate_pct="$(
    awk -v socket="$socket_error_total" -v total="$total_with_socket_errors" \
      'BEGIN { if (total <= 0) { print "100.000000" } else { printf "%.6f", (socket * 100.0) / total } }'
  )"
  if awk -v observed="$socket_error_rate_pct" -v max="$socket_error_max_rate_pct" 'BEGIN { exit !(observed <= max) }'; then
    echo "warning: workbench profile socketErrors tolerated: impl=${impl} endpoint=${endpoint} socketErrors=${socket_error_total} ratePct=${socket_error_rate_pct} maxRatePct=${socket_error_max_rate_pct}" >&2
  else
    echo "workbench profile failed: impl=${impl} endpoint=${endpoint} socketErrors=${socket_error_total} ratePct=${socket_error_rate_pct} maxRatePct=${socket_error_max_rate_pct}" >&2
    exit 1
  fi
fi
