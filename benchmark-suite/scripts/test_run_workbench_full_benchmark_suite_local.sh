#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
cat >"$tmp_env" <<EOF
SEC4_RT_LASM_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
EOF

out="$($root_dir/scripts/run_workbench_full_benchmark_suite_local.sh \
  --dry-run \
  --infra-env "$tmp_env" \
  --impls sec4-lasm \
  --endpoints wb-task-get \
  --port 18115)"

if ! grep -q '^local postgres workbench full mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing local workbench full dry-run marker" >&2
  exit 1
fi
if ! grep -q 'delegating: .*run_workbench_full_benchmark_suite.sh --lasm-db-adapter postgres --lasm-postgres-dsn-file .* --dry-run --impls sec4-lasm --endpoints wb-task-get --port 18115' <<<"$out"; then
  echo "missing delegated local workbench full command marker" >&2
  exit 1
fi
if ! grep -q '^preflight passed$' <<<"$out"; then
  echo "missing preflight pass output from delegated full suite" >&2
  exit 1
fi
if ! grep -q 'start: impl=sec4-lasm servicePath=benchmark-suite/services/sec4-lasm-workbench port=18115 lasmDbAdapter=postgres lasmPostgresDsn=' <<<"$out"; then
  echo "missing sec4-lasm postgres adapter start marker in local full dry-run output" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_full_benchmark_suite_local.sh" --dry-run --impls sec4 --endpoints wb-task-get --lasm-db-adapter sqlite >/dev/null 2>&1; then
  echo "expected local wrapper conflicting --lasm-db-adapter to fail" >&2
  exit 1
fi

echo "run_workbench_full_benchmark_suite_local test passed"
