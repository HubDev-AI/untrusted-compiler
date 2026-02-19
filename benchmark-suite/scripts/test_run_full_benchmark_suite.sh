#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/run_full_benchmark_suite.sh --dry-run --impls node --endpoints ping)"

if ! grep -q '^phase: fixed-target matrix$' <<<"$out"; then
  echo "missing fixed-target phase" >&2
  exit 1
fi
if ! grep -q '^phase: step-load matrix$' <<<"$out"; then
  echo "missing step-load phase" >&2
  exit 1
fi
if ! grep -q 'run_profile.sh --dry-run node ping' <<<"$out"; then
  echo "missing delegated fixed-target dry-run command" >&2
  exit 1
fi
if ! grep -q 'run_step_profile.sh --dry-run node ping' <<<"$out"; then
  echo "missing delegated step-load dry-run command" >&2
  exit 1
fi
if ! grep -q 'publish_report.sh .*compare-matrix.json .*benchmark-report.md .*analysis.json .*step-matrix.json' <<<"$out"; then
  echo "missing combined publish command" >&2
  exit 1
fi
if ! grep -q 'build_artifact_manifest.sh .*results .*artifact-manifest.json' <<<"$out"; then
  echo "missing artifact manifest command" >&2
  exit 1
fi

out_sat="$($root_dir/scripts/run_full_benchmark_suite.sh --dry-run --impls sec4-lasm --endpoints ping --include-lasm-saturation --saturation-skip-verify)"
if ! grep -q '^phase: lasm saturation tuning bundle$' <<<"$out_sat"; then
  echo "missing lasm saturation phase" >&2
  exit 1
fi
if ! grep -q '^sec4 LASM saturation boost bundle plan:$' <<<"$out_sat"; then
  echo "missing delegated saturation bundle plan output" >&2
  exit 1
fi
if ! grep -q 'publish_report.sh .*compare-matrix.json .*benchmark-report.md .*analysis.json .*step-matrix.json .*sec4-lasm-cluster-saturation-boost-summary.md' <<<"$out_sat"; then
  echo "missing publish command with saturation summary input" >&2
  exit 1
fi

tuned_sat="$($root_dir/scripts/run_full_benchmark_suite.sh --dry-run --impls sec4-lasm --endpoints ping --include-lasm-saturation --saturation-boost-steps 3,5 --saturation-project-path examples/hello --saturation-duration 55s --saturation-threads 3 --saturation-connections 99 --saturation-target-requests 12345 --saturation-cluster-relay-workers 11 --saturation-cluster-relay-queue 222 --saturation-skip-verify)"
if ! grep -q 'projectPath=examples/hello' <<<"$tuned_sat"; then
  echo "missing delegated saturation project-path override" >&2
  exit 1
fi
if ! grep -q 'duration=55s' <<<"$tuned_sat"; then
  echo "missing delegated saturation duration override" >&2
  exit 1
fi
if ! grep -q 'threads=3' <<<"$tuned_sat"; then
  echo "missing delegated saturation threads override" >&2
  exit 1
fi
if ! grep -q 'connections=99' <<<"$tuned_sat"; then
  echo "missing delegated saturation connections override" >&2
  exit 1
fi
if ! grep -q 'targetRequests=12345' <<<"$tuned_sat"; then
  echo "missing delegated saturation target-requests override" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=11' <<<"$tuned_sat"; then
  echo "missing delegated saturation relay-workers override" >&2
  exit 1
fi
if ! grep -q 'clusterRelayQueue=222' <<<"$tuned_sat"; then
  echo "missing delegated saturation relay-queue override" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=3' <<<"$tuned_sat"; then
  echo "missing delegated saturation boost-step 3 run" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=5' <<<"$tuned_sat"; then
  echo "missing delegated saturation boost-step 5 run" >&2
  exit 1
fi

if "$root_dir/scripts/run_full_benchmark_suite.sh" --dry-run --impls unknown --endpoints ping >/dev/null 2>&1; then
  echo "expected invalid impl to fail via delegated validation" >&2
  exit 1
fi
if "$root_dir/scripts/run_full_benchmark_suite.sh" --dry-run --impls node --endpoints ping --include-lasm-saturation >/tmp/run-full-sat-invalid.log 2>&1; then
  echo "expected include-lasm-saturation without sec4-lasm to fail" >&2
  exit 1
fi
if ! grep -q -- '--include-lasm-saturation requires sec4-lasm in --impls' /tmp/run-full-sat-invalid.log; then
  echo "missing include-lasm-saturation guard diagnostic" >&2
  exit 1
fi

echo "run_full_benchmark_suite test passed"
