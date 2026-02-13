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

if "$root_dir/scripts/run_full_benchmark_suite.sh" --dry-run --impls unknown --endpoints ping >/dev/null 2>&1; then
  echo "expected invalid impl to fail via delegated validation" >&2
  exit 1
fi

echo "run_full_benchmark_suite test passed"
