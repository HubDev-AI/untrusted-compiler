#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_dsn="$(mktemp)"
tmp_out="$(mktemp)"
trap 'rm -f "$tmp_dsn" "$tmp_out"' EXIT
printf '%s\n' 'postgresql://bench:bench@127.0.0.1:5432/bench' >"$tmp_dsn"

out="$($root_dir/scripts/run_alpha_postgres_comparison_suite_repeats.sh \
  --dry-run \
  --runs 2 \
  --base-impls sec4-lasm \
  --base-endpoints ping \
  --db-impls sec4-lasm \
  --db-endpoints db-hot-write \
  --lasm-db-postgres-dsn-file "$tmp_dsn" \
  --out "$tmp_out")"

if ! grep -q '^repeat-run: 1/2 (run-001)$' <<<"$out"; then
  echo "missing run-001 marker" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 2/2 (run-002)$' <<<"$out"; then
  echo "missing run-002 marker" >&2
  exit 1
fi
if ! grep -q 'run_alpha_postgres_comparison_suite.sh --dry-run --base-impls sec4-lasm --base-endpoints ping --db-impls sec4-lasm --db-endpoints db-hot-write --lasm-db-postgres-dsn-file' <<<"$out"; then
  echo "missing delegated suite command marker" >&2
  exit 1
fi
if ! grep -q "wrote ${tmp_out}" <<<"$out"; then
  echo "missing aggregate summary write marker" >&2
  exit 1
fi

if [ ! -f "$tmp_out" ]; then
  echo "expected output summary file to exist: $tmp_out" >&2
  exit 1
fi

run_count="$(jq -r '.runCount' "$tmp_out")"
dry_run="$(jq -r '.dryRun' "$tmp_out")"
runs_len="$(jq -r '.runs | length' "$tmp_out")"
first_run="$(jq -r '.runs[0].run' "$tmp_out")"
first_dry="$(jq -r '.runs[0].dryRun' "$tmp_out")"

if [ "$run_count" != "2" ]; then
  echo "unexpected runCount: $run_count" >&2
  exit 1
fi
if [ "$dry_run" != "true" ]; then
  echo "unexpected dryRun field: $dry_run" >&2
  exit 1
fi
if [ "$runs_len" != "2" ]; then
  echo "unexpected runs length: $runs_len" >&2
  exit 1
fi
if [ "$first_run" != "run-001" ]; then
  echo "unexpected first run marker: $first_run" >&2
  exit 1
fi
if [ "$first_dry" != "true" ]; then
  echo "expected first run dryRun=true, got: $first_dry" >&2
  exit 1
fi

echo "run_alpha_postgres_comparison_suite_repeats test passed"
