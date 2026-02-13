#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/run_step_matrix.sh --dry-run --impls node,go --endpoints ping,decode)"

if ! grep -q '^preflight passed$' <<<"$out"; then
  echo "missing preflight pass output" >&2
  exit 1
fi
if ! grep -q '=== step impl=node ===' <<<"$out"; then
  echo "missing node step impl header" >&2
  exit 1
fi
if ! grep -q 'run_step_profile.sh --dry-run node ping' <<<"$out"; then
  echo "missing node ping step dry-run command" >&2
  exit 1
fi
if ! grep -q 'analyze_step_profile.sh .*node-ping-step.json .*node-ping-step-analysis.json' <<<"$out"; then
  echo "missing node ping step analysis command" >&2
  exit 1
fi
if ! grep -q 'compare_step_matrix.sh' <<<"$out"; then
  echo "missing step matrix compare command" >&2
  exit 1
fi

if "$root_dir/scripts/run_step_matrix.sh" --dry-run --impls unknown --endpoints ping >/dev/null 2>&1; then
  echo "expected unsupported impl to fail" >&2
  exit 1
fi
if "$root_dir/scripts/run_step_matrix.sh" --dry-run --impls node --endpoints unknown >/dev/null 2>&1; then
  echo "expected unsupported endpoint to fail" >&2
  exit 1
fi

echo "run_step_matrix test passed"
