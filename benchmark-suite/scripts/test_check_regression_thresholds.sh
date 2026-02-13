#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
matrix="$root_dir/testdata/sample-compare-matrix.json"
baseline="$root_dir/testdata/sample-trend-baseline-ping.json"

"$root_dir/check_regression_thresholds.sh" "$matrix" --endpoint ping --max-p99-ms 10 --min-target-coverage 90 >/dev/null
"$root_dir/check_regression_thresholds.sh" "$matrix" --endpoint ping --baseline "$baseline" >/dev/null

if "$root_dir/check_regression_thresholds.sh" "$matrix" --endpoint ping --max-p99-ms 1 --min-target-coverage 90 >/dev/null 2>&1; then
  echo "expected p99 threshold failure" >&2
  exit 1
fi

if "$root_dir/check_regression_thresholds.sh" "$matrix" --endpoint ping --max-p99-ms 10 --min-target-coverage 99 >/dev/null 2>&1; then
  echo "expected target coverage threshold failure" >&2
  exit 1
fi

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

strict_baseline="$tmp/strict-baseline.json"
jq '.baselineP99Ms = 3.0 | .maxP99RegressionPct = 0' "$baseline" > "$strict_baseline"
if "$root_dir/check_regression_thresholds.sh" "$matrix" --endpoint ping --baseline "$strict_baseline" >/dev/null 2>&1; then
  echo "expected baseline p99 regression failure" >&2
  exit 1
fi

echo "check_regression_thresholds test passed"
