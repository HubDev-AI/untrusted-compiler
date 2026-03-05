#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_dsn="$(mktemp)"
trap 'rm -f "$tmp_dsn"' EXIT
printf '%s\n' 'postgresql://bench:bench@127.0.0.1:5432/bench' >"$tmp_dsn"

out="$($root_dir/scripts/run_alpha_postgres_comparison_suite.sh --dry-run --base-impls sec4-lasm --base-endpoints ping --db-impls sec4-lasm --db-endpoints db-hot-write --lasm-db-postgres-dsn-file "$tmp_dsn")"
out_reset="$($root_dir/scripts/run_alpha_postgres_comparison_suite.sh --dry-run --reset-db-between-phases --base-impls sec4-lasm --base-endpoints ping --db-impls sec4-lasm --db-endpoints db-hot-write --lasm-db-postgres-dsn-file "$tmp_dsn")"

if ! grep -q '^phase: alpha postgres comparison suite (baseline)$' <<<"$out"; then
  echo "missing baseline phase marker" >&2
  exit 1
fi
if ! grep -q '^phase: alpha postgres comparison suite (db-hot)$' <<<"$out"; then
  echo "missing db-hot phase marker" >&2
  exit 1
fi
if ! grep -q 'run_full_benchmark_suite.sh --impls sec4-lasm --endpoints ping --lasm-db-adapter postgres' <<<"$out"; then
  echo "missing baseline delegated run_full command" >&2
  exit 1
fi
if ! grep -q 'run_full_benchmark_suite.sh --impls sec4-lasm --endpoints db-hot-write --lasm-db-adapter postgres' <<<"$out"; then
  echo "missing db-hot delegated run_full command" >&2
  exit 1
fi
if ! grep -q "start: sec4-lasm service on :18085 (db-adapter=postgres dsn-file=${tmp_dsn})" <<<"$out"; then
  echo "missing postgres adapter start marker in delegated dry-run output" >&2
  exit 1
fi
if ! grep -q 'snapshot: compare-matrix.json -> .*compare-matrix-alpha-base.json' <<<"$out"; then
  echo "missing baseline snapshot plan" >&2
  exit 1
fi
if ! grep -q 'snapshot: compare-matrix.json -> .*compare-matrix-alpha-db-postgres.json' <<<"$out"; then
  echo "missing db-hot snapshot plan" >&2
  exit 1
fi
if ! grep -q '^reset-db (between-phases): would drop benchmark tables via psql$' <<<"$out_reset"; then
  echo "missing reset-db dry-run marker" >&2
  exit 1
fi

if "$root_dir/scripts/run_alpha_postgres_comparison_suite.sh" --dry-run --base-impls sec4-lasm --base-endpoints ping --db-impls sec4-lasm --db-endpoints db-hot-write >/dev/null 2>&1; then
  echo "expected missing postgres dsn source to fail" >&2
  exit 1
fi

echo "run_alpha_postgres_comparison_suite test passed"
