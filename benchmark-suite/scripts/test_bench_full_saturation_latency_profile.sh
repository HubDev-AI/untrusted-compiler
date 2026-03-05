#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$(make -C "${root_dir}" bench-full-saturation-latency-dry 2>&1)"

if ! grep -q '^phase: lasm saturation tuning bundle$' <<<"${out}"; then
  echo "missing latency preset saturation phase output" >&2
  exit 1
fi
if ! grep -q 'boostSteps=1,2,3' <<<"${out}"; then
  echo "missing latency preset boost-step defaults" >&2
  exit 1
fi
if ! grep -q 'duration=30s' <<<"${out}"; then
  echo "missing latency preset duration default" >&2
  exit 1
fi
if ! grep -q 'threads=4' <<<"${out}"; then
  echo "missing latency preset threads default" >&2
  exit 1
fi
if ! grep -q 'connections=64' <<<"${out}"; then
  echo "missing latency preset connections default" >&2
  exit 1
fi
if ! grep -q 'targetRequests=250000' <<<"${out}"; then
  echo "missing latency preset request-floor default" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=16' <<<"${out}"; then
  echo "missing latency preset relay-worker default" >&2
  exit 1
fi
if ! grep -q 'clusterRelayQueue=2048' <<<"${out}"; then
  echo "missing latency preset relay-queue default" >&2
  exit 1
fi
if ! grep -q 'clusterRelayPumpBatchMax=auto' <<<"${out}"; then
  echo "missing latency preset relay-pump-batch default marker" >&2
  exit 1
fi

echo "bench_full_saturation_latency_profile test passed"
