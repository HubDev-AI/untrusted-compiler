#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

matrix_pass="$tmp/saturation-matrix-pass.json"
cat >"${matrix_pass}" <<'JSON'
{
  "impl": "sec4-lasm-cluster",
  "run": {
    "targetRequests": 1000000
  },
  "boostSteps": [2, 4, 6],
  "runs": [
    {
      "saturationBoostStep": 2,
      "pass": true,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 12000,
      "summaryFile": "probe-2.json"
    },
    {
      "saturationBoostStep": 4,
      "pass": true,
      "requests": 1260000,
      "requestsPerSec": 63000,
      "peakRssKb": 12500,
      "summaryFile": "probe-4.json"
    },
    {
      "saturationBoostStep": 6,
      "pass": false,
      "requests": 1400000,
      "requestsPerSec": 70000,
      "peakRssKb": 11000,
      "summaryFile": "probe-6.json"
    }
  ]
}
JSON

analysis_pass="$tmp/analysis-pass.json"
"${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh" "${matrix_pass}" "${analysis_pass}" >/dev/null

if ! jq -e '.summary.selectionMode == "pass-first"' "${analysis_pass}" >/dev/null; then
  echo "analysis should use pass-first mode when pass runs exist" >&2
  exit 1
fi
if ! jq -e '.summary.recommendedBoostStep == 4' "${analysis_pass}" >/dev/null; then
  echo "analysis chose wrong recommended boost step in pass-first mode" >&2
  exit 1
fi
if ! jq -e '.rankedRuns | map(.saturationBoostStep) == [4, 2, 6]' "${analysis_pass}" >/dev/null; then
  echo "analysis ranking order mismatch for pass-first mode" >&2
  exit 1
fi

matrix_no_pass="$tmp/saturation-matrix-no-pass.json"
cat >"${matrix_no_pass}" <<'JSON'
{
  "impl": "sec4-lasm-cluster",
  "run": {
    "targetRequests": 1000000
  },
  "boostSteps": [2, 4, 6],
  "runs": [
    {
      "saturationBoostStep": 2,
      "pass": false,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 10000,
      "summaryFile": "probe-2.json"
    },
    {
      "saturationBoostStep": 4,
      "pass": false,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 9000,
      "summaryFile": "probe-4.json"
    },
    {
      "saturationBoostStep": 6,
      "pass": false,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 9000,
      "summaryFile": "probe-6.json"
    }
  ]
}
JSON

analysis_no_pass="$tmp/analysis-no-pass.json"
"${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh" "${matrix_no_pass}" "${analysis_no_pass}" >/dev/null

if ! jq -e '.summary.selectionMode == "throughput-best-no-pass"' "${analysis_no_pass}" >/dev/null; then
  echo "analysis should report no-pass selection mode when all runs fail" >&2
  exit 1
fi
if ! jq -e '.summary.recommendedBoostStep == 4' "${analysis_no_pass}" >/dev/null; then
  echo "analysis should tie-break by lower rss then lower boost step" >&2
  exit 1
fi
if ! jq -e '.rankedRuns | map(.saturationBoostStep) == [4, 6, 2]' "${analysis_no_pass}" >/dev/null; then
  echo "analysis ranking order mismatch for no-pass tie-breaks" >&2
  exit 1
fi

matrix_p99_tie="$tmp/saturation-matrix-p99-tie.json"
cat >"${matrix_p99_tie}" <<'JSON'
{
  "impl": "sec4-lasm-cluster",
  "runs": [
    {
      "saturationBoostStep": 2,
      "pass": true,
      "requests": 1250000,
      "requestsPerSec": 62500,
      "peakRssKb": 12000,
      "p99": "6.10ms"
    },
    {
      "saturationBoostStep": 4,
      "pass": true,
      "requests": 1250000,
      "requestsPerSec": 62500,
      "peakRssKb": 12000,
      "p99": "5.40ms"
    }
  ]
}
JSON

analysis_p99_tie="$tmp/analysis-p99-tie.json"
"${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh" "${matrix_p99_tie}" "${analysis_p99_tie}" >/dev/null
if ! jq -e '.summary.recommendedBoostStep == 4' "${analysis_p99_tie}" >/dev/null; then
  echo "analysis should prefer lower p99 when throughput and pass state tie" >&2
  exit 1
fi
if ! jq -e '.rankedRuns | map(.saturationBoostStep) == [4, 2]' "${analysis_p99_tie}" >/dev/null; then
  echo "analysis ranking order mismatch for p99 tie-break mode" >&2
  exit 1
fi

matrix_empty="$tmp/saturation-matrix-empty.json"
cat >"${matrix_empty}" <<'JSON'
{
  "runs": []
}
JSON

if "${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh" "${matrix_empty}" "${tmp}/analysis-empty.json" >/tmp/lasm-sat-boost-analysis-empty.log 2>&1; then
  echo "analysis accepted an empty runs array" >&2
  exit 1
fi
if ! grep -q 'matrix runs must be a non-empty array' /tmp/lasm-sat-boost-analysis-empty.log; then
  echo "analysis missing empty runs diagnostic" >&2
  exit 1
fi

matrix_invalid="$tmp/saturation-matrix-invalid.json"
cat >"${matrix_invalid}" <<'JSON'
{
  "runs": [
    {
      "saturationBoostStep": 2,
      "pass": true,
      "requests": 1200000,
      "requestsPerSec": "fast",
      "peakRssKb": 10000
    }
  ]
}
JSON

if "${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh" "${matrix_invalid}" "${tmp}/analysis-invalid.json" >/tmp/lasm-sat-boost-analysis-invalid.log 2>&1; then
  echo "analysis accepted invalid run field types" >&2
  exit 1
fi
if ! grep -q 'matrix runs contain invalid fields' /tmp/lasm-sat-boost-analysis-invalid.log; then
  echo "analysis missing invalid-field diagnostic" >&2
  exit 1
fi

echo "analyze_lasm_cluster_saturation_boost_matrix test passed"
