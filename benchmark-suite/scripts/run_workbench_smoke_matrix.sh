#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
suite_dir="$(cd "$script_dir/.." && pwd)"
repo_root="$(cd "$suite_dir/.." && pwd)"

matrix_path="${1:-$suite_dir/workbench/matrix.backends.json}"
out_path="${2:-$suite_dir/results/summaries/workbench-smoke-matrix.json}"

if [ ! -f "$matrix_path" ]; then
  echo "workbench matrix missing: $matrix_path" >&2
  exit 1
fi

mkdir -p "$(dirname "$out_path")"

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
results_json='[]'
totals_passed=0
totals_failed=0
totals_skipped=0

while IFS= read -r impl_row; do
  impl="$(jq -r '.impl' <<<"$impl_row")"
  status="$(jq -r '.status' <<<"$impl_row")"
  service_rel="$(jq -r '.servicePath' <<<"$impl_row")"
  service_abs="$repo_root/$service_rel"
  smoke_script="$service_abs/smoke.sh"

  result_status="skipped"
  reason=""
  exit_code=0

  if [ "$status" != "implemented-alpha" ] && [ "$status" != "implemented" ]; then
    reason="status=$status"
    totals_skipped=$((totals_skipped + 1))
  elif [ ! -x "$smoke_script" ]; then
    reason="missing executable smoke script: $smoke_script"
    totals_skipped=$((totals_skipped + 1))
  else
    if (cd "$repo_root" && "$smoke_script"); then
      result_status="passed"
      totals_passed=$((totals_passed + 1))
    else
      result_status="failed"
      exit_code=$?
      reason="smoke script exited with code $exit_code"
      totals_failed=$((totals_failed + 1))
    fi
  fi

  row_json="$(jq -nc \
    --arg impl "$impl" \
    --arg status "$status" \
    --arg servicePath "$service_rel" \
    --arg result "$result_status" \
    --arg reason "$reason" \
    --argjson exitCode "$exit_code" \
    '{
      impl: $impl,
      status: $status,
      servicePath: $servicePath,
      smokeResult: $result,
      reason: (if $reason == "" then null else $reason end),
      exitCode: $exitCode
    }')"
  results_json="$(jq -c --argjson row "$row_json" '. + [$row]' <<<"$results_json")"
done < <(jq -c '.implementations[]' "$matrix_path")

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
  --arg version "0.1" \
  --arg startedAt "$started_at" \
  --arg finishedAt "$finished_at" \
  --arg matrixPath "$matrix_path" \
  --argjson totals "$(jq -nc \
    --argjson passed "$totals_passed" \
    --argjson failed "$totals_failed" \
    --argjson skipped "$totals_skipped" \
    '{passed:$passed,failed:$failed,skipped:$skipped}')" \
  --argjson runs "$results_json" \
  '{
    version: $version,
    startedAt: $startedAt,
    finishedAt: $finishedAt,
    matrixPath: $matrixPath,
    totals: $totals,
    runs: $runs
  }' >"$out_path"

echo "wrote workbench smoke matrix: $out_path"
echo "totals: passed=$totals_passed failed=$totals_failed skipped=$totals_skipped"

if [ "$totals_failed" -gt 0 ]; then
  exit 1
fi
