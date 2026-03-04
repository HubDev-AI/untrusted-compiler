#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_env="$(mktemp)"
tmp_summary="$(mktemp)"
tmp_report="$(mktemp)"
trap 'rm -f "$tmp_env" "$tmp_summary" "$tmp_report"' EXIT
cat >"$tmp_env" <<EOF
SEC4_DB_ALPHA_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
SEC4_RT_LASM_DB_POSTGRES_DSN=postgresql://bench:bench@127.0.0.1:5432/bench
EOF

out="$($root_dir/scripts/run_workbench_full_benchmark_suite_local_bundle.sh \
  --dry-run \
  --runs 2 \
  --out-summary "$tmp_summary" \
  --out-report "$tmp_report" \
  --infra-env "$tmp_env" \
  --impls sec4 \
  --endpoints wb-task-get \
  --port 18117)"

if ! grep -q '^run: .*run_workbench_full_benchmark_suite_local_repeats.sh --runs 2 --out '"$tmp_summary"' --dry-run --infra-env '"$tmp_env"' --impls sec4 --endpoints wb-task-get --port 18117$' <<<"$out"; then
  echo "missing delegated local bundle run command marker" >&2
  exit 1
fi
if ! grep -q '^local postgres workbench full repeats mode: dry-run (no docker start/stop)$' <<<"$out"; then
  echo "missing delegated local repeats dry-run marker from bundle output" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 1/2 (run-001)$' <<<"$out"; then
  echo "missing run-001 marker in local bundle dry-run output" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 2/2 (run-002)$' <<<"$out"; then
  echo "missing run-002 marker in local bundle dry-run output" >&2
  exit 1
fi
if ! grep -q '^run: .*render_workbench_full_benchmark_suite_repeats_summary.sh '"$tmp_summary"' '"$tmp_report"'$' <<<"$out"; then
  echo "missing dry-run render command marker from local bundle output" >&2
  exit 1
fi
if [ ! -s "$tmp_summary" ]; then
  echo "missing local bundle dry-run summary output" >&2
  exit 1
fi
if [ -s "$tmp_report" ]; then
  echo "dry-run local bundle should not render markdown report" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_full_benchmark_suite_local_bundle.sh" --dry-run --runs 0 --impls sec4 --endpoints wb-task-get >/dev/null 2>&1; then
  echo "expected --runs 0 to fail for local bundle wrapper" >&2
  exit 1
fi

echo "run_workbench_full_benchmark_suite_local_bundle test passed"
