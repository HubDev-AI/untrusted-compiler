#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$(make -C "${root_dir}" bench-full-saturation-throughput-dry 2>&1)"

if ! grep -q '^phase: lasm saturation tuning bundle$' <<<"${out}"; then
  echo "missing throughput preset saturation phase output" >&2
  exit 1
fi
if ! grep -q 'boostSteps=2,4,6,8' <<<"${out}"; then
  echo "missing throughput preset boost-step defaults" >&2
  exit 1
fi
if ! grep -q 'duration=60s' <<<"${out}"; then
  echo "missing throughput preset duration default" >&2
  exit 1
fi
if ! grep -q 'threads=12' <<<"${out}"; then
  echo "missing throughput preset threads default" >&2
  exit 1
fi
if ! grep -q 'connections=512' <<<"${out}"; then
  echo "missing throughput preset connections default" >&2
  exit 1
fi
if ! grep -q 'targetRequests=1000000' <<<"${out}"; then
  echo "missing throughput preset request-floor default" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=32' <<<"${out}"; then
  echo "missing throughput preset relay-worker default" >&2
  exit 1
fi
if ! grep -q 'clusterRelayQueue=4096' <<<"${out}"; then
  echo "missing throughput preset relay-queue default" >&2
  exit 1
fi

echo "bench_full_saturation_throughput_profile test passed"
