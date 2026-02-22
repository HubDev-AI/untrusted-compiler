#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" \
  --dry-run \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 7s \
  --threads 2 \
  --connections 16 \
  --target-requests 4567 \
  --port 19094 \
  --instances 3 \
  --autoscale-max-instances 5 \
  --autoscale-target-connections 111 \
  --autoscale-check-ms 250 \
  --autoscale-scale-up-cooldown-ms 123 \
  --autoscale-scale-down-cooldown-ms 456 \
  --autoscale-scale-up-step 5 \
  --autoscale-scale-down-step 2 \
  --autoscale-saturation-boost-step 6 \
  --cluster-relay-workers 9 \
  --cluster-relay-queue 999 \
  --cluster-accept-workers 4 \
  --cluster-relay-accept-batch-max 321 \
  --cluster-relay-pump-batch-max 654 \
  --build-profile debug \
  --samples 3 \
  --wrk-processes 4 \
  --proxy-out results/summaries/custom-lasm-mode-compare-proxy.json \
  --fixed-out results/summaries/custom-lasm-mode-compare-fixed.json \
  --out results/summaries/custom-lasm-mode-compare.json \
  2>&1)"

if ! grep -q 'sec4 LASM cluster mode compare plan:' <<<"$out"; then
  echo "mode compare dry-run missing plan header" >&2
  exit 1
fi
if ! grep -q "proxyOut=${root_dir}/results/summaries/custom-lasm-mode-compare-proxy.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved proxy output path" >&2
  exit 1
fi
if ! grep -q "fixedOut=${root_dir}/results/summaries/custom-lasm-mode-compare-fixed.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved fixed output path" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/custom-lasm-mode-compare.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved comparison output path" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=9' <<<"$out"; then
  echo "mode compare dry-run missing proxy relay workers passthrough" >&2
  exit 1
fi
if ! grep -q 'buildProfile=debug' <<<"$out"; then
  echo "mode compare dry-run missing build profile marker" >&2
  exit 1
fi
if ! grep -q 'samples=3' <<<"$out"; then
  echo "mode compare dry-run missing samples marker" >&2
  exit 1
fi
if ! grep -q 'wrkProcesses=4' <<<"$out"; then
  echo "mode compare dry-run missing wrk-processes marker" >&2
  exit 1
fi
if ! grep -q 'fixedReusePortMode=true' <<<"$out"; then
  echo "mode compare dry-run missing delegated fixed reuse-port mode marker" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --request-header invalid >/tmp/lasm-mode-compare-invalid-header.log 2>&1; then
  echo "mode compare accepted invalid request header" >&2
  exit 1
fi
if ! grep -q "request-header must include ':'" /tmp/lasm-mode-compare-invalid-header.log; then
  echo "mode compare missing invalid request-header diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --cluster-relay-pump-batch-max nope >/tmp/lasm-mode-compare-invalid-pump.log 2>&1; then
  echo "mode compare accepted invalid relay pump batch value" >&2
  exit 1
fi
if ! grep -q "cluster-relay-pump-batch-max must be numeric" /tmp/lasm-mode-compare-invalid-pump.log; then
  echo "mode compare missing invalid relay pump batch diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --samples 0 >/tmp/lasm-mode-compare-invalid-samples.log 2>&1; then
  echo "mode compare accepted invalid samples value" >&2
  exit 1
fi
if ! grep -q "samples must be >= 1" /tmp/lasm-mode-compare-invalid-samples.log; then
  echo "mode compare missing invalid samples diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --wrk-processes 0 >/tmp/lasm-mode-compare-invalid-wrk-processes.log 2>&1; then
  echo "mode compare accepted invalid wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be >= 1" /tmp/lasm-mode-compare-invalid-wrk-processes.log; then
  echo "mode compare missing invalid wrk-processes diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --wrk-processes nope >/tmp/lasm-mode-compare-invalid-wrk-processes-type.log 2>&1; then
  echo "mode compare accepted non-integer wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be an integer >= 1" /tmp/lasm-mode-compare-invalid-wrk-processes-type.log; then
  echo "mode compare missing invalid wrk-processes type diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --build-profile fast >/tmp/lasm-mode-compare-invalid-profile.log 2>&1; then
  echo "mode compare accepted invalid build profile value" >&2
  exit 1
fi
if ! grep -q "build-profile must be one of: debug, release" /tmp/lasm-mode-compare-invalid-profile.log; then
  echo "mode compare missing invalid build profile diagnostic" >&2
  exit 1
fi

echo "run_lasm_cluster_mode_compare test passed"
