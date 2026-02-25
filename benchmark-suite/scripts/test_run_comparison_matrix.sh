#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls=node,go)"

if ! grep -q '^preflight passed$' <<<"$out"; then
  echo "missing preflight pass output" >&2
  exit 1
fi

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

if ! grep -q 'run_profile.sh --dry-run node users-get' <<<"$out"; then
  echo "missing node users-get dry-run command" >&2
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
if ! grep -q 'compare_matrix.sh .* node,go' <<<"$out"; then
  echo "missing scoped impl list in compare matrix command" >&2
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

ping_only_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls=node --endpoints=ping)"
if ! grep -q 'run_profile.sh --dry-run node ping' <<<"$ping_only_out"; then
  echo "missing ping command in endpoint-filtered dry-run" >&2
  exit 1
fi
if grep -q 'run_profile.sh --dry-run node decode' <<<"$ping_only_out"; then
  echo "unexpected decode command in endpoint-filtered dry-run" >&2
  exit 1
fi
if ! grep -q 'build_report.sh node .* "" ping' <<<"$ping_only_out"; then
  echo "missing endpoint-filtered report command for ping-only run" >&2
  exit 1
fi

c_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls c)"
if ! grep -q '=== impl=c ===' <<<"$c_out"; then
  echo "missing c implementation header" >&2
  exit 1
fi

sec4_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls sec4)"
if ! grep -q '=== impl=sec4 ===' <<<"$sec4_out"; then
  echo "missing sec4 implementation header" >&2
  exit 1
fi
if ! grep -q 'build_report.sh sec4 .*baselines/sec-audit/default-secure-prod.hello.json' <<<"$sec4_out"; then
  echo "expected sec4 report command to include sec4 audit artifact" >&2
  exit 1
fi

sec4_lasm_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls sec4-lasm)"
if ! grep -q '=== impl=sec4-lasm ===' <<<"$sec4_lasm_out"; then
  echo "missing sec4-lasm implementation header" >&2
  exit 1
fi
if ! grep -q 'build_report.sh sec4-lasm .*baselines/sec-audit/default-secure-prod.hello.json' <<<"$sec4_lasm_out"; then
  echo "expected sec4-lasm report command to include sec4 audit artifact" >&2
  exit 1
fi

tmp_dsn="$(mktemp)"
trap 'rm -f "$tmp_dsn"' EXIT
printf '%s\n' 'postgresql://bench:bench@127.0.0.1:5432/bench' >"$tmp_dsn"
lasm_db_out="$($root_dir/scripts/run_comparison_matrix.sh --dry-run --impls sec4-lasm --endpoints db-hot-write,db-hot-query-one --lasm-db-adapter postgres --lasm-db-postgres-dsn-file "$tmp_dsn")"
if ! grep -q "start: sec4-lasm service on :18085 (db-adapter=postgres dsn-file=${tmp_dsn})" <<<"$lasm_db_out"; then
  echo "missing sec4-lasm postgres adapter start marker in dry-run output" >&2
  exit 1
fi
if ! grep -q 'run_profile.sh --dry-run sec4-lasm db-hot-write' <<<"$lasm_db_out"; then
  echo "missing sec4-lasm db-hot-write dry-run command" >&2
  exit 1
fi
if ! grep -q 'run_profile.sh --dry-run sec4-lasm db-hot-query-one' <<<"$lasm_db_out"; then
  echo "missing sec4-lasm db-hot-query-one dry-run command" >&2
  exit 1
fi

if "$root_dir/scripts/run_comparison_matrix.sh" --dry-run --impls unknown >/dev/null 2>&1; then
  echo "expected unsupported implementation to fail" >&2
  exit 1
fi

echo "run_comparison_matrix test passed"
