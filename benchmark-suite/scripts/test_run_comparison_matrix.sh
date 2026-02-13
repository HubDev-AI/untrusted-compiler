#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls node,go)"

if ! grep -q '=== impl=node ===' <<<"$out"; then
  echo "missing node implementation header" >&2
  exit 1
fi

if ! grep -q 'run_profile.sh --dry-run node ping' <<<"$out"; then
  echo "missing node ping dry-run command" >&2
  exit 1
fi

if ! grep -q 'run_profile.sh --dry-run go decode' <<<"$out"; then
  echo "missing go decode dry-run command" >&2
  exit 1
fi

if ! grep -q 'build_report.sh go' <<<"$out"; then
  echo "missing go report command" >&2
  exit 1
fi

if ! grep -q 'compare_matrix.sh' <<<"$out"; then
  echo "missing compare matrix command" >&2
  exit 1
fi

if ! grep -q 'analyze_matrix.sh' <<<"$out"; then
  echo "missing matrix analysis command" >&2
  exit 1
fi

if ! grep -q 'publish_report.sh' <<<"$out"; then
  echo "missing publish report command" >&2
  exit 1
fi

c_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls c)"
if ! grep -q '=== impl=c ===' <<<"$c_out"; then
  echo "missing c implementation header" >&2
  exit 1
fi

ailang_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls ailang)"
if ! grep -q '=== impl=ailang ===' <<<"$ailang_out"; then
  echo "missing ailang implementation header" >&2
  exit 1
fi

if "$root_dir/scripts/run_comparison_matrix.sh" --dry-run --impls unknown >/dev/null 2>&1; then
  echo "expected unsupported implementation to fail" >&2
  exit 1
fi

echo "run_comparison_matrix test passed"
