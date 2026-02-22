#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" \
  --dry-run \
  --boost-steps 2,4,7 \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 6s \
  --threads 2 \
  --connections 32 \
  --target-requests 12345 \
  --autoscale-target-connections 111 \
  --autoscale-scale-up-step 2 \
  --autoscale-scale-down-step 1 \
  --build-profile debug \
  --samples 3 \
  --cluster-accept-workers 3 \
  --cluster-relay-accept-batch-max 123 \
  --cluster-relay-pump-batch-max 456 \
  --out results/summaries/custom-saturation-boost-matrix.json \
  --analysis-out results/summaries/custom-saturation-boost-analysis.json \
  --verify-recommended \
  --verify-out results/summaries/custom-saturation-boost-verify.json \
  2>&1)"

if ! grep -q 'sec4 LASM saturation boost matrix plan:' <<<"$out"; then
  echo "saturation boost matrix dry-run missing plan header" >&2
  exit 1
fi
if ! grep -q 'boostSteps=2,4,7' <<<"$out"; then
  echo "saturation boost matrix dry-run missing boost steps listing" >&2
  exit 1
fi
if ! grep -q '=== saturationBoostStep=2 ===' <<<"$out"; then
  echo "saturation boost matrix dry-run missing step 2 section header" >&2
  exit 1
fi
if ! grep -q '=== saturationBoostStep=7 ===' <<<"$out"; then
  echo "saturation boost matrix dry-run missing step 7 section header" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=2' <<<"$out"; then
  echo "saturation boost matrix dry-run missing probe output for step 2" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=7' <<<"$out"; then
  echo "saturation boost matrix dry-run missing probe output for step 7" >&2
  exit 1
fi
if ! grep -q 'clusterAcceptWorkers=3' <<<"$out"; then
  echo "saturation boost matrix dry-run missing accept workers override output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayAcceptBatchMax=123' <<<"$out"; then
  echo "saturation boost matrix dry-run missing relay accept batch override output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayPumpBatchMax=456' <<<"$out"; then
  echo "saturation boost matrix dry-run missing relay pump batch override output" >&2
  exit 1
fi
if ! grep -q 'buildProfile=debug' <<<"$out"; then
  echo "saturation boost matrix dry-run missing build profile output" >&2
  exit 1
fi
if ! grep -q 'samples=3' <<<"$out"; then
  echo "saturation boost matrix dry-run missing samples output" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-7.json" <<<"$out"; then
  echo "saturation boost matrix dry-run missing resolved per-step output path" >&2
  exit 1
fi
if ! grep -q "analysisOut=${root_dir}/results/summaries/custom-saturation-boost-analysis.json" <<<"$out"; then
  echo "saturation boost matrix dry-run missing analysis output path" >&2
  exit 1
fi
if ! grep -q "analysisCmd=${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh ${root_dir}/results/summaries/custom-saturation-boost-matrix.json ${root_dir}/results/summaries/custom-saturation-boost-analysis.json" <<<"$out"; then
  echo "saturation boost matrix dry-run missing analysis command plan" >&2
  exit 1
fi
if ! grep -q 'verifyRecommended=true' <<<"$out"; then
  echo "saturation boost matrix dry-run missing verify-recommended marker" >&2
  exit 1
fi
if ! grep -q "verifyOut=${root_dir}/results/summaries/custom-saturation-boost-verify.json" <<<"$out"; then
  echo "saturation boost matrix dry-run missing verify output path" >&2
  exit 1
fi
if ! grep -q 'verifyRecommendedAfterAnalysis=true' <<<"$out"; then
  echo "saturation boost matrix dry-run missing verify-after-analysis marker" >&2
  exit 1
fi
if ! grep -q "verifyCmd=${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh ... --autoscale-saturation-boost-step <recommended> --build-profile debug --samples 3 --out ${root_dir}/results/summaries/custom-saturation-boost-verify.json --skip-build" <<<"$out"; then
  echo "saturation boost matrix dry-run missing verify command plan" >&2
  exit 1
fi

out_skip_analysis="$("${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" \
  --dry-run \
  --skip-analysis \
  --boost-steps 2,4 \
  2>&1)"
if ! grep -q 'skipAnalysis=true' <<<"$out_skip_analysis"; then
  echo "saturation boost matrix dry-run missing skip-analysis plan marker" >&2
  exit 1
fi
if grep -q 'analysisCmd=' <<<"$out_skip_analysis"; then
  echo "saturation boost matrix dry-run should not print analysis command when skip-analysis is set" >&2
  exit 1
fi

out_fixed="$("${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" \
  --dry-run \
  --fixed-reuse-port-mode \
  --boost-steps 2,4 \
  --instances 3 \
  --autoscale-max-instances 9 \
  2>&1)"
if ! grep -q 'fixedReusePortMode=true' <<<"$out_fixed"; then
  echo "saturation boost matrix dry-run missing fixed reuse-port mode marker" >&2
  exit 1
fi
if ! grep -q 'autoscaleMaxInstances=3' <<<"$out_fixed"; then
  echo "saturation boost matrix dry-run missing delegated fixed-mode autoscale max override" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" --dry-run --skip-analysis --verify-recommended >/tmp/lasm-sat-boost-matrix-invalid-verify.log 2>&1; then
  echo "saturation boost matrix accepted verify-recommended with skip-analysis" >&2
  exit 1
fi
if ! grep -q 'verify-recommended requires analysis; remove --skip-analysis' /tmp/lasm-sat-boost-matrix-invalid-verify.log; then
  echo "saturation boost matrix missing verify-without-analysis diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" --dry-run --boost-steps 2,abc >/tmp/lasm-sat-boost-matrix-invalid-shape.log 2>&1; then
  echo "saturation boost matrix accepted invalid non-numeric boost step" >&2
  exit 1
fi
if ! grep -q 'boost-steps must contain positive integers, got: abc' /tmp/lasm-sat-boost-matrix-invalid-shape.log; then
  echo "saturation boost matrix missing invalid-shape diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" --dry-run --boost-steps 2,0 >/tmp/lasm-sat-boost-matrix-invalid-range.log 2>&1; then
  echo "saturation boost matrix accepted zero boost step" >&2
  exit 1
fi
if ! grep -q 'boost-steps must be >= 1, got: 0' /tmp/lasm-sat-boost-matrix-invalid-range.log; then
  echo "saturation boost matrix missing invalid-range diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" --dry-run --samples 0 >/tmp/lasm-sat-boost-matrix-invalid-samples.log 2>&1; then
  echo "saturation boost matrix accepted invalid samples value" >&2
  exit 1
fi
if ! grep -q 'samples must be >= 1, got: 0' /tmp/lasm-sat-boost-matrix-invalid-samples.log; then
  echo "saturation boost matrix missing invalid samples diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh" --dry-run --build-profile fast >/tmp/lasm-sat-boost-matrix-invalid-profile.log 2>&1; then
  echo "saturation boost matrix accepted invalid build profile value" >&2
  exit 1
fi
if ! grep -q 'build-profile must be one of: debug, release' /tmp/lasm-sat-boost-matrix-invalid-profile.log; then
  echo "saturation boost matrix missing invalid build profile diagnostic" >&2
  exit 1
fi

echo "run_lasm_cluster_saturation_boost_matrix test passed"
