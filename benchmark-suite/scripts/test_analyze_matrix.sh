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
if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .metrics.constantRate == true' "$analysis" >/dev/null; then
  echo "analysis expected constantRate=true for sample matrix" >&2
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

# verify p99 unit normalization to milliseconds (us/ms/s)
units_matrix="$tmp/units-matrix.json"
cat >"$units_matrix" <<'JSON'
{
  "endpoints": [
    {
      "endpoint": "mixed-units",
      "leader": {
        "impl": "sec4",
        "endpoint": "mixed-units",
        "targetRps": 100,
        "requestsPerSec": 100,
        "p99": "500us",
        "loadGenerator": "wrk2",
        "constantRate": true
      },
      "compared": [
        {"impl":"sec4","endpoint":"mixed-units","targetRps":100,"requestsPerSec":100,"p99":"500us","loadGenerator":"wrk2","constantRate":true},
        {"impl":"go","endpoint":"mixed-units","targetRps":100,"requestsPerSec":95,"p99":"2ms","loadGenerator":"wrk2","constantRate":true},
        {"impl":"node","endpoint":"mixed-units","targetRps":100,"requestsPerSec":90,"p99":"2500us","loadGenerator":"wrk2","constantRate":true},
        {"impl":"rust","endpoint":"mixed-units","targetRps":100,"requestsPerSec":80,"p99":"0.5s","loadGenerator":"wrk2","constantRate":true}
      ]
    }
  ]
}
JSON

units_analysis="$tmp/units-analysis.json"
"$root_dir/scripts/analyze_matrix.sh" "$units_matrix" "$units_analysis" >/dev/null

if ! jq -e '.endpoints[] | select(.endpoint == "mixed-units") | .metrics.p99MinMs == 0.5' "$units_analysis" >/dev/null; then
  echo "expected p99MinMs to normalize 500us -> 0.5ms" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "mixed-units") | .metrics.p99MaxMs == 500' "$units_analysis" >/dev/null; then
  echo "expected p99MaxMs to normalize 0.5s -> 500ms" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "mixed-units") | .metrics.p99SpreadX == 1000' "$units_analysis" >/dev/null; then
  echo "expected p99SpreadX to use normalized millisecond units" >&2
  exit 1
fi

echo "analyze_matrix test passed"
