#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--dry-run] <impl> <endpoint:wb-tasks-post|wb-tasks-with-comment|wb-task-comment-post|wb-task-get|wb-tasks-list> [base_url]" >&2
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

step_rates="${BENCH_STEP_RATES:-}"
if [ -z "$step_rates" ]; then
  case "$endpoint" in
    wb-tasks-post) step_rates="200,500,800" ;;
    wb-tasks-with-comment) step_rates="120,300,500" ;;
    wb-task-comment-post) step_rates="200,500,800" ;;
    wb-task-get) step_rates="800,1500,2500" ;;
    wb-tasks-list) step_rates="500,1000,1500" ;;
    *)
      echo "unsupported workbench endpoint: $endpoint" >&2
      usage
      exit 2
      ;;
  esac
fi

step_duration="${BENCH_STEP_DURATION:-30s}"

IFS=',' read -r -a rates <<<"$step_rates"
if [ "${#rates[@]}" -eq 0 ]; then
  echo "no workbench step rates configured" >&2
  exit 2
fi

summary_files=()
for raw_rate in "${rates[@]}"; do
  rate="${raw_rate// /}"
  [ -z "$rate" ] && continue

  tagged_raw="${raw_dir}/${impl}-${endpoint}-r${rate}.txt"
  tagged_summary="${sum_dir}/${impl}-${endpoint}-r${rate}.json"

  echo "workbench step profile impl=${impl} endpoint=${endpoint} rate=${rate} duration=${step_duration}"
  if [ "$dry_run" = "true" ]; then
    echo "run: BENCH_DURATION=${step_duration} BENCH_TARGET=${rate} ${root_dir}/scripts/run_workbench_profile.sh --dry-run ${impl} ${endpoint} ${base_url}"
    echo "write: ${tagged_raw}"
    echo "write: ${tagged_summary}"
    continue
  fi

  BENCH_SERVER_PID="${BENCH_SERVER_PID:-}" \
    BENCH_WB_TASK_ID="${BENCH_WB_TASK_ID:-}" \
    BENCH_WB_RUN_TAG="${BENCH_WB_RUN_TAG:-}" \
    BENCH_DURATION="$step_duration" \
    BENCH_TARGET="$rate" \
    "${root_dir}/scripts/run_workbench_profile.sh" "$impl" "$endpoint" "$base_url"
  cp "${raw_dir}/${impl}-${endpoint}.txt" "$tagged_raw"
  cp "${sum_dir}/${impl}-${endpoint}.json" "$tagged_summary"
  summary_files+=("$tagged_summary")
done

if [ "$dry_run" = "true" ]; then
  exit 0
fi

if [ "${#summary_files[@]}" -eq 0 ]; then
  echo "no workbench step summaries produced for impl=${impl} endpoint=${endpoint}" >&2
  exit 1
fi

step_out="${sum_dir}/${impl}-${endpoint}-step.json"
jq -n \
  --arg impl "$impl" \
  --arg endpoint "$endpoint" \
  --arg rates "$step_rates" \
  --arg duration "$step_duration" \
  --argjson steps "$(jq -s '.' "${summary_files[@]}")" \
  '{
    version: "0.1",
    impl: $impl,
    endpoint: $endpoint,
    stepRates: ($rates | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
    stepDuration: $duration,
    steps: $steps
  }' >"$step_out"

echo "wrote ${step_out}"
