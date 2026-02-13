#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="${root_dir}/results/summaries/test-summary.json"

"${root_dir}/scripts/wrk2_summary.sh" \
  "${root_dir}/scripts/testdata/wrk2_sample.txt" \
  "ailang" \
  "ping" \
  "10000" \
  "$out" >/dev/null

if ! grep -q '"impl": "ailang"' "$out"; then
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

echo "wrk2 summary test passed"
