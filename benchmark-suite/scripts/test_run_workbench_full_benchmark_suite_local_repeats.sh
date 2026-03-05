#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
cat >"$tmp_env" <<EOF
SEC4_DB_ALPHA_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
SEC4_RT_LASM_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
EOF

out="$($root_dir/scripts/run_workbench_full_benchmark_suite_local_repeats.sh \
  --dry-run \
  --infra-env "$tmp_env" \
  --runs 2 \
  --impls sec4 \
  --endpoints wb-task-get \
  --port 18116)"

if ! grep -q '^local postgres workbench full repeats mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing local workbench full repeats dry-run marker" >&2
  exit 1
fi
if ! grep -q 'delegating: .*run_workbench_full_benchmark_suite_repeats.sh --lasm-db-adapter postgres --lasm-postgres-dsn-file .* --dry-run --runs 2 --impls sec4 --endpoints wb-task-get --port 18116' <<<"$out"; then
  echo "missing delegated local workbench full repeats command marker" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 1/2 (run-001)$' <<<"$out"; then
  echo "missing run-001 marker from delegated local full repeats suite" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 2/2 (run-002)$' <<<"$out"; then
  echo "missing run-002 marker from delegated local full repeats suite" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_full_benchmark_suite_local_repeats.sh" --dry-run --runs 1 --impls sec4 --endpoints wb-task-get --lasm-postgres-dsn-file /tmp/x >/dev/null 2>&1; then
  echo "expected local repeats wrapper conflicting --lasm-postgres-dsn-file to fail" >&2
  exit 1
fi

echo "run_workbench_full_benchmark_suite_local_repeats test passed"
