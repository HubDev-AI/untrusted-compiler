#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="${root_dir}/results/summaries/test-summary.json"

"${root_dir}/scripts/wrk2_summary.sh" \
  "${root_dir}/scripts/testdata/wrk2_sample.txt" \
  "sec4" \
  "ping" \
  "10000" \
  "$out" \
  "4242" \
  "ps" >/dev/null

if ! grep -q '"impl": "sec4"' "$out"; then
  echo "missing impl in summary" >&2
  exit 1
fi
if ! grep -q '"endpoint": "ping"' "$out"; then
  echo "missing endpoint in summary" >&2
  exit 1
fi
if ! grep -q '"requestsPerSec": 9993.73' "$out"; then
  echo "missing requestsPerSec in summary" >&2
  exit 1
fi
if ! grep -q '"p95": "3.20ms"' "$out"; then
  echo "missing p95 in summary" >&2
  exit 1
fi
if ! grep -q '"loadGenerator": "wrk2"' "$out"; then
  echo "missing wrk2 loadGenerator marker in summary" >&2
  exit 1
fi
if ! grep -q '"constantRate": true' "$out"; then
  echo "missing constantRate=true in wrk2 summary" >&2
  exit 1
fi
if ! grep -q '"rssKb": 4242' "$out"; then
  echo "missing rssKb in wrk2 summary" >&2
  exit 1
fi
if ! grep -q '"sampleSource": "ps"' "$out"; then
  echo "missing memory sample source in wrk2 summary" >&2
  exit 1
fi

fallback_out="${root_dir}/results/summaries/test-summary-wrk.json"
"${root_dir}/scripts/wrk2_summary.sh" \
  "${root_dir}/scripts/testdata/wrk_sample.txt" \
  "go" \
  "decode" \
  "2000" \
  "$fallback_out" >/dev/null

if ! grep -q '"impl": "go"' "$fallback_out"; then
  echo "missing impl in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"requestsPerSec": 124864.23' "$fallback_out"; then
  echo "missing requestsPerSec in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"p50": "343.00us"' "$fallback_out"; then
  echo "missing p50 in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"p99": "2.04ms"' "$fallback_out"; then
  echo "missing p99 in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"loadGenerator": "wrk"' "$fallback_out"; then
  echo "missing wrk loadGenerator marker in fallback summary" >&2
  exit 1
fi
if ! grep -q '"constantRate": false' "$fallback_out"; then
  echo "missing constantRate=false in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"rssKb": null' "$fallback_out"; then
  echo "missing null rssKb in wrk fallback summary" >&2
  exit 1
fi
if ! grep -q '"sampleSource": "unavailable"' "$fallback_out"; then
  echo "missing unavailable memory sample source in wrk fallback summary" >&2
  exit 1
fi

echo "wrk2 summary test passed"
