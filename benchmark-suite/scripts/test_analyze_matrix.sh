#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"
cp "$root_dir/scripts/testdata/sample-rust-report.json" "$tmp/rust-report.json"

matrix="$tmp/compare-matrix.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$matrix" >/dev/null

analysis="$tmp/analysis.json"
"$root_dir/scripts/analyze_matrix.sh" "$matrix" "$analysis" >/dev/null

if ! jq -e '.version == "0.1"' "$analysis" >/dev/null; then
  echo "analysis missing version" >&2
  exit 1
fi

if ! jq -e '.summary.endpointCount == 3' "$analysis" >/dev/null; then
  echo "analysis endpoint count mismatch" >&2
  exit 1
fi

if ! jq -e '.summary.highestSeverity == "MEDIUM"' "$analysis" >/dev/null; then
  echo "analysis highest severity mismatch" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .findings[] | select(.id == "P99_SPREAD_MEDIUM")' "$analysis" >/dev/null; then
  echo "expected ping p99 spread finding" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "users-post") | (.findings | length) == 1' "$analysis" >/dev/null; then
  echo "expected users-post coverage warning finding" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "users-post") | .findings[] | select(.id == "LEADER_TARGET_COVERAGE_WARN")' "$analysis" >/dev/null; then
  echo "expected users-post target coverage warning" >&2
  exit 1
fi

echo "analyze_matrix test passed"
