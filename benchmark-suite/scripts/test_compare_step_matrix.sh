#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

"$root_dir/scripts/analyze_step_profile.sh" \
  "$root_dir/scripts/testdata/sample-step-summary-decode.json" \
  "$tmp/ailang-decode-step-analysis.json" \
  "0.91" >/dev/null

jq '.impl = "go" | .summary.kneeAtTargetRps = 3000 | .summary.kneeObservedRps = 2500 | .summary.achievedRatioMin = 0.78' \
  "$tmp/ailang-decode-step-analysis.json" > "$tmp/go-decode-step-analysis.json"

jq '.impl = "node" | .endpoint = "ping" | .summary.kneeDetected = false | .summary.kneeAtTargetRps = 0 | .summary.kneeObservedRps = 0' \
  "$tmp/ailang-decode-step-analysis.json" > "$tmp/node-ping-step-analysis.json"

out="$tmp/step-matrix.json"
"$root_dir/scripts/compare_step_matrix.sh" "$tmp" "$out" >/dev/null

if ! jq -e '.version == "0.1"' "$out" >/dev/null; then
  echo "step matrix missing version" >&2
  exit 1
fi

if ! jq -e '.endpoints | length == 2' "$out" >/dev/null; then
  echo "step matrix expected 2 endpoints" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "decode") | .leader.impl == "go"' "$out" >/dev/null; then
  echo "step matrix expected go to lead decode" >&2
  exit 1
fi

filtered_out="$tmp/step-matrix-filtered.json"
"$root_dir/scripts/compare_step_matrix.sh" "$tmp" "$filtered_out" "ailang,go" "decode" >/dev/null

if ! jq -e '.endpoints | length == 1' "$filtered_out" >/dev/null; then
  echo "filtered step matrix expected one endpoint" >&2
  exit 1
fi
if ! jq -e '.summary.implementationCount == 2' "$filtered_out" >/dev/null; then
  echo "filtered step matrix expected two implementations" >&2
  exit 1
fi

if "$root_dir/scripts/compare_step_matrix.sh" "$tmp" "$out" "ailang,missing" "decode" >/dev/null 2>&1; then
  echo "expected missing selected step analysis to fail" >&2
  exit 1
fi

echo "compare_step_matrix test passed"
