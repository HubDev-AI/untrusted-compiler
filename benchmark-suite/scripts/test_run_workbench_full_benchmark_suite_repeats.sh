#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

out="$($root_dir/scripts/run_workbench_full_benchmark_suite_repeats.sh --dry-run --runs 2 --impls sec4 --endpoints wb-task-get --port 18112 --out "$tmp_out")"

if ! grep -q '^repeat-run: 1/2 (run-001)$' <<<"$out"; then
  echo "missing run-001 marker in repeats dry-run output" >&2
  exit 1
fi
if ! grep -q '^repeat-run: 2/2 (run-002)$' <<<"$out"; then
  echo "missing run-002 marker in repeats dry-run output" >&2
  exit 1
fi
if ! grep -q 'run_workbench_full_benchmark_suite.sh --dry-run --impls sec4 --endpoints wb-task-get --port 18112 --out-runs .*/workbench-full-run-001.json' <<<"$out"; then
  echo "missing delegated run-001 full-suite command in repeats output" >&2
  exit 1
fi

if [ ! -f "$tmp_out" ]; then
  echo "missing repeats summary output file" >&2
  exit 1
fi
if ! jq -e '.mode == "workbench-full-benchmark-suite-repeats" and .dryRun == true and .runCount == 2 and (.runs | length) == 2 and .compareStats == null and .stepStats == null' "$tmp_out" >/dev/null; then
  echo "invalid repeats summary contract for dry-run output" >&2
  exit 1
fi
if ! jq -e '
  (.scripts.wrapper | startswith("benchmark-suite/scripts/"))
  and (.scripts.inner | startswith("benchmark-suite/scripts/"))
  and ([.runs[].summary, .runs[].fixedRuns, .runs[].stepRuns, .runs[].compareMatrix, .runs[].analysis, .runs[].stepMatrix, .runs[].report]
    | all(startswith("benchmark-suite/results/summaries/workbench-full-benchmark-runs/")))
' "$tmp_out" >/dev/null; then
  echo "expected repeats summary artifact references to be repo-relative" >&2
  exit 1
fi

if "$root_dir/scripts/run_workbench_full_benchmark_suite_repeats.sh" --runs 0 --dry-run --impls sec4 --endpoints wb-task-get >/dev/null 2>&1; then
  echo "expected --runs 0 to fail" >&2
  exit 1
fi

echo "run_workbench_full_benchmark_suite_repeats test passed"
