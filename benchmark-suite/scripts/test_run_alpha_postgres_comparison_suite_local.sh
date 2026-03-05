#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/run_alpha_postgres_comparison_suite_local.sh --dry-run --base-impls sec4-lasm --base-endpoints ping --db-impls sec4-lasm --db-endpoints db-hot-write)"

if ! grep -q '^local postgres mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing local dry-run marker" >&2
  exit 1
fi

if ! grep -q '^phase: alpha postgres comparison suite (baseline)$' <<<"$out"; then
  echo "missing baseline phase marker" >&2
  exit 1
fi

if ! grep -q '^phase: alpha postgres comparison suite (db-hot)$' <<<"$out"; then
  echo "missing db-hot phase marker" >&2
  exit 1
fi

if ! grep -q 'run_full_benchmark_suite.sh --impls sec4-lasm --endpoints ping --lasm-db-adapter postgres' <<<"$out"; then
  echo "missing delegated baseline command" >&2
  exit 1
fi

if ! grep -q 'run_full_benchmark_suite.sh --impls sec4-lasm --endpoints db-hot-write --lasm-db-adapter postgres' <<<"$out"; then
  echo "missing delegated db-hot command" >&2
  exit 1
fi

if ! grep -q 'db-adapter=postgres dsn-file=' <<<"$out"; then
  echo "missing delegated postgres dsn-file marker" >&2
  exit 1
fi

echo "run_alpha_postgres_comparison_suite_local test passed"
