#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

out="$tmp/analysis.json"
"$root_dir/scripts/analyze_step_profile.sh" "$root_dir/scripts/testdata/sample-step-summary-decode.json" "$out" "0.91" >/dev/null

if ! jq -e '.version == "0.1"' "$out" >/dev/null; then
  echo "missing version in step analysis" >&2
  exit 1
fi

if ! jq -e '.summary.kneeDetected == true' "$out" >/dev/null; then
  echo "expected knee detection to be true" >&2
  exit 1
fi

if ! jq -e '.summary.kneeAtTargetRps == 2000' "$out" >/dev/null; then
  echo "expected knee at target rps 2000" >&2
  exit 1
fi

if ! jq -e '.summary.p99MaxMs == 24.5' "$out" >/dev/null; then
  echo "expected p99 max ms extraction" >&2
  exit 1
fi

if "$root_dir/scripts/analyze_step_profile.sh" "$root_dir/scripts/testdata/does-not-exist.json" "$out" >/dev/null 2>&1; then
  echo "expected missing input to fail" >&2
  exit 1
fi

echo "analyze_step_profile test passed"
