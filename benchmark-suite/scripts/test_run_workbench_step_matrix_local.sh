#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
cat >"$tmp_env" <<EOF
SEC4_RT_LASM_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
EOF

out="$($root_dir/scripts/run_workbench_step_matrix_local.sh \
  --dry-run \
  --infra-env "$tmp_env" \
  --impls sec4-lasm \
  --endpoints wb-task-get \
  --port 18119)"

if ! grep -q '^local postgres workbench step mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing local workbench step dry-run marker" >&2
  exit 1
fi
if ! grep -q 'delegating: .*run_workbench_step_matrix.sh --lasm-db-adapter postgres --lasm-postgres-dsn-file .* --dry-run --impls sec4-lasm --endpoints wb-task-get --port 18119' <<<"$out"; then
  echo "missing delegated local workbench step command marker" >&2
  exit 1
fi
if ! grep -q 'run_workbench_step_profile.sh --dry-run sec4-lasm wb-task-get' <<<"$out"; then
  echo "missing delegated step profile dry-run command marker" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_step_matrix_local.sh" --dry-run --impls sec4 --endpoints wb-task-get --lasm-postgres-dsn-file /tmp/x >/dev/null 2>&1; then
  echo "expected local workbench step wrapper conflicting --lasm-postgres-dsn-file to fail" >&2
  exit 1
fi

echo "run_workbench_step_matrix_local test passed"
