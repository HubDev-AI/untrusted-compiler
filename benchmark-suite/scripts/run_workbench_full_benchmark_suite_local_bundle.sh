#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--runs 3] [--dry-run] [--out-summary path] [--out-report path]
          [local-repeats-args...]

Runs benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_repeats.sh
and (non-dry-run) renders markdown summary in one command.

Wrapper options:
  --runs <n>        Number of repeated runs (default: 3)
  --dry-run         Forward dry-run to local repeated suite; skip markdown render.
  --out-summary     Repeated-suite summary path (default: benchmark-suite/results/summaries/workbench-full-benchmark-repeats.json)
  --out-report      Markdown summary path (default: benchmark-suite/results/workbench-full-benchmark-repeats.md)

All other arguments are forwarded to run_workbench_full_benchmark_suite_local_repeats.sh.
USAGE
}

runs="3"
dry_run="false"
out_summary=""
out_report=""
passthrough=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      runs="$2"
      shift 2
      ;;
    --runs=*)
      runs="${1#--runs=}"
      shift
      ;;
    --out-summary)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_summary="$2"
      shift 2
      ;;
    --out-summary=*)
      out_summary="${1#--out-summary=}"
      shift
      ;;
    --out-report)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_report="$2"
      shift 2
      ;;
    --out-report=*)
      out_report="${1#--out-report=}"
      shift
      ;;
    --dry-run)
      dry_run="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      passthrough+=("$1")
      shift
      ;;
  esac
done

case "$runs" in
  ''|*[!0-9]*)
    echo "--runs must be an integer >= 1" >&2
    exit 2
    ;;
esac
if [ "$runs" -lt 1 ]; then
  echo "--runs must be >= 1" >&2
  exit 2
fi

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"
if [ -z "$out_summary" ]; then
  out_summary="${suite_dir}/results/summaries/workbench-full-benchmark-repeats.json"
fi
if [ -z "$out_report" ]; then
  out_report="${suite_dir}/results/workbench-full-benchmark-repeats.md"
fi

mkdir -p "$(dirname "$out_summary")" "$(dirname "$out_report")"

run_cmd=(
  "${suite_dir}/scripts/run_workbench_full_benchmark_suite_local_repeats.sh"
  --runs "$runs"
  --out "$out_summary"
)
if [ "$dry_run" = "true" ]; then
  run_cmd+=(--dry-run)
fi
run_cmd+=("${passthrough[@]}")

echo "run: ${run_cmd[*]}"
"${run_cmd[@]}"

render_cmd=(
  "${suite_dir}/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh"
  "$out_summary"
  "$out_report"
)

if [ "$dry_run" = "true" ]; then
  echo "run: ${render_cmd[*]}"
  exit 0
fi

"${render_cmd[@]}"
