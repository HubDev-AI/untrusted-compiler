#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
cat >"$tmp_env" <<EOF
SEC4_RT_LASM_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
EOF

out="$($root_dir/scripts/run_alpha_postgres_comparison_suite_local_repeats.sh \
  --dry-run \
  --infra-env "$tmp_env" \
  --runs 2 \
  --base-impls sec4-lasm \
  --base-endpoints ping \
  --db-impls sec4-lasm \
  --db-endpoints db-hot-write)"

if ! grep -q '^local postgres repeats mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing local repeats dry-run marker" >&2
  exit 1
fi
if ! grep -q 'delegating: .*run_alpha_postgres_comparison_suite_repeats.sh --lasm-db-postgres-dsn-file .* --dry-run --runs 2' <<<"$out"; then
  echo "missing delegated repeated-suite command marker" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 1/2 (run-001)$' <<<"$out"; then
  echo "missing run-001 marker from delegated repeated suite" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 2/2 (run-002)$' <<<"$out"; then
  echo "missing run-002 marker from delegated repeated suite" >&2
  exit 1
fi
if ! grep -q 'alpha-postgres-comparison-suite-repeats.json' <<<"$out"; then
  echo "missing repeated-suite summary write marker" >&2
  exit 1
fi

echo "run_alpha_postgres_comparison_suite_local_repeats test passed"
