#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_dsn="$(mktemp)"
trap 'rm -f "$tmp_dsn"' EXIT
printf '%s\n' 'postgresql://bench:bench@127.0.0.1:5432/bench' >"$tmp_dsn"

out="$($root_dir/scripts/run_workbench_full_benchmark_suite.sh --dry-run --impls sec4 --endpoints wb-task-get --port 18110)"

if ! grep -q '^preflight passed$' <<<"$out"; then
  echo "missing preflight pass output" >&2
  exit 1
fi
if ! grep -q 'run_workbench_profile.sh --dry-run sec4 wb-task-get' <<<"$out"; then
  echo "missing fixed-target workbench profile dry-run command" >&2
  exit 1
fi
if ! grep -q 'run_workbench_step_profile.sh --dry-run sec4 wb-task-get' <<<"$out"; then
  echo "missing step-load workbench profile dry-run command" >&2
  exit 1
fi
if ! grep -q 'publish_report.sh .*workbench-benchmark-compare-matrix.json .*workbench-full-benchmark-report.md .*workbench-benchmark-analysis.json .*workbench-step-matrix.json' <<<"$out"; then
  echo "missing combined publish command with step matrix" >&2
  exit 1
fi

db_out="$($root_dir/scripts/run_workbench_full_benchmark_suite.sh --dry-run --impls sec4-lasm --endpoints wb-task-get --lasm-db-adapter postgres --lasm-postgres-dsn-file "$tmp_dsn" --port 18111)"
if ! grep -q "start: impl=sec4-lasm servicePath=benchmark-suite/services/sec4-lasm-workbench port=18111 lasmDbAdapter=postgres lasmPostgresDsn=${tmp_dsn}" <<<"$db_out"; then
  echo "missing sec4-lasm postgres adapter start marker in workbench full dry-run output" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_full_benchmark_suite.sh" --dry-run --impls sec4 --endpoints unknown >/dev/null 2>&1; then
  echo "expected unsupported endpoint to fail" >&2
  exit 1
fi

echo "run_workbench_full_benchmark_suite test passed"
