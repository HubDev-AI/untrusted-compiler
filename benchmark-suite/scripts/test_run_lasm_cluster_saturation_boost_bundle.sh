#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_saturation_boost_bundle.sh" \
  --dry-run \
  --boost-steps 2,4,7 \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 6s \
  --threads 2 \
  --connections 32 \
  --target-requests 12345 \
  --cluster-accept-workers 5 \
  --cluster-relay-accept-batch-max 222 \
  --cluster-relay-pump-batch-max 333 \
  --summary-out results/summaries/custom-saturation-boost-summary.md \
  2>&1)"

if ! grep -q 'sec4 LASM saturation boost bundle plan:' <<<"$out"; then
  echo "saturation boost bundle dry-run missing bundle plan header" >&2
  exit 1
fi
if ! grep -q 'verifyRecommended=true' <<<"$out"; then
  echo "saturation boost bundle dry-run missing default verify mode marker" >&2
  exit 1
fi
if ! grep -q 'sec4 LASM saturation boost matrix plan:' <<<"$out"; then
  echo "saturation boost bundle dry-run missing matrix plan passthrough" >&2
  exit 1
fi
if ! grep -q 'clusterAcceptWorkers=5' <<<"$out"; then
  echo "saturation boost bundle dry-run missing accept workers passthrough" >&2
  exit 1
fi
if ! grep -q 'clusterRelayAcceptBatchMax=222' <<<"$out"; then
  echo "saturation boost bundle dry-run missing relay accept batch passthrough" >&2
  exit 1
fi
if ! grep -q 'clusterRelayPumpBatchMax=333' <<<"$out"; then
  echo "saturation boost bundle dry-run missing relay pump batch passthrough" >&2
  exit 1
fi
if ! grep -q "summaryCmd=${root_dir}/scripts/render_lasm_cluster_saturation_boost_summary.sh ${root_dir}/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json ${root_dir}/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json ${root_dir}/results/summaries/custom-saturation-boost-summary.md ${root_dir}/results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json" <<<"$out"; then
  echo "saturation boost bundle dry-run missing summary command plan with verify artifact" >&2
  exit 1
fi

out_skip_verify="$("${root_dir}/scripts/run_lasm_cluster_saturation_boost_bundle.sh" \
  --dry-run \
  --skip-verify \
  --summary-out results/summaries/custom-saturation-boost-summary-no-verify.md \
  2>&1)"
if ! grep -q 'verifyRecommended=false' <<<"$out_skip_verify"; then
  echo "saturation boost bundle dry-run missing skip-verify mode marker" >&2
  exit 1
fi
if ! grep -q "summaryCmd=${root_dir}/scripts/render_lasm_cluster_saturation_boost_summary.sh ${root_dir}/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json ${root_dir}/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json ${root_dir}/results/summaries/custom-saturation-boost-summary-no-verify.md" <<<"$out_skip_verify"; then
  echo "saturation boost bundle dry-run missing summary command plan without verify artifact" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_bundle.sh" --dry-run --boost-steps 2,abc >/tmp/lasm-sat-boost-bundle-invalid-shape.log 2>&1; then
  echo "saturation boost bundle accepted invalid boost-step shape" >&2
  exit 1
fi
if ! grep -q 'boost-steps must contain positive integers, got: abc' /tmp/lasm-sat-boost-bundle-invalid-shape.log; then
  echo "saturation boost bundle missing invalid-shape diagnostic passthrough" >&2
  exit 1
fi

echo "run_lasm_cluster_saturation_boost_bundle test passed"
