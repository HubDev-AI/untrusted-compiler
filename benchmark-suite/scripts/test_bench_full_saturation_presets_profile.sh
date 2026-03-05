#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$(make -C "${root_dir}" bench-full-saturation-presets-dry 2>&1)"

if ! grep -q 'boostSteps=2,4,6,8' <<<"${out}"; then
  echo "missing throughput preset execution in combined presets target" >&2
  exit 1
fi
if ! grep -q 'boostSteps=1,2,3' <<<"${out}"; then
  echo "missing latency preset execution in combined presets target" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=32' <<<"${out}"; then
  echo "missing throughput relay defaults in combined presets target" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=16' <<<"${out}"; then
  echo "missing latency relay defaults in combined presets target" >&2
  exit 1
fi
if [ "$(grep -c 'clusterRelayPumpBatchMax=auto' <<<"${out}")" -lt 2 ]; then
  echo "missing relay-pump-batch default marker in combined presets target" >&2
  exit 1
fi

echo "bench_full_saturation_presets_profile test passed"
