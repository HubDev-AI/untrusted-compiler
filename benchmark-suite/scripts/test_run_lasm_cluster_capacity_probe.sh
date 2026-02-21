#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" \
  --dry-run \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 7s \
  --threads 2 \
  --connections 16 \
  --target-requests 4567 \
  --port 19091 \
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
  --out results/summaries/custom-lasm-capacity.json \
  2>&1)"

if ! grep -q 'projectPath=' <<<"$out"; then
  echo "lasm capacity probe dry-run missing project path output" >&2
  exit 1
fi
if ! grep -q 'requestPath=/health' <<<"$out"; then
  echo "lasm capacity probe dry-run missing requestPath output" >&2
  exit 1
fi
if ! grep -q 'targetRequests=4567' <<<"$out"; then
  echo "lasm capacity probe dry-run missing targetRequests output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleUpCooldownMs=123' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-up cooldown output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleDownCooldownMs=456' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-down cooldown output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleUpStep=5' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-up step output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleDownStep=2' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-down step output" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=6' <<<"$out"; then
  echo "lasm capacity probe dry-run missing saturation boost step output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=9' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay workers output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayQueue=999' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay queue output" >&2
  exit 1
fi
if ! grep -q 'clusterAcceptWorkers=4' <<<"$out"; then
  echo "lasm capacity probe dry-run missing accept workers output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayAcceptBatchMax=321' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay accept batch output" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/custom-lasm-capacity.json" <<<"$out"; then
  echo "lasm capacity probe dry-run missing resolved output path" >&2
  exit 1
fi
if ! grep -q "clusterStatusJson=${root_dir}/results/raw/sec4-lasm-cluster-capacity-status-19091.json" <<<"$out"; then
  echo "lasm capacity probe dry-run missing resolved cluster status json path" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --request-header invalid >/tmp/lasm-capacity-probe-invalid.log 2>&1; then
  echo "lasm capacity probe accepted invalid request header" >&2
  exit 1
fi
if ! grep -q "request-header must include ':'" /tmp/lasm-capacity-probe-invalid.log; then
  echo "lasm capacity probe invalid request-header error missing" >&2
  exit 1
fi

echo "run_lasm_cluster_capacity_probe test passed"
