#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--runs 3] [--dry-run] [--out path] [<workbench-full args...>]

Runs benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh repeatedly
under identical arguments, writes run-scoped artifacts, and emits one aggregate
run-manifest JSON.

Wrapper options:
  --runs <n>     Number of repeated runs (default: 3)
  --dry-run      Forward dry-run to inner suite
  --out <path>   Aggregate summary output path

All other arguments are forwarded to run_workbench_full_benchmark_suite.sh.
USAGE
}

runs="3"
dry_run="false"
out_path=""
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
    --out)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_path="$2"
      shift 2
      ;;
    --out=*)
      out_path="${1#--out=}"
      shift
      ;;
    --dry-run)
      dry_run="true"
      passthrough+=("--dry-run")
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
summaries_dir="${suite_dir}/results/summaries"
runs_dir="${summaries_dir}/workbench-full-benchmark-runs"
mkdir -p "$summaries_dir" "$runs_dir"

if [ -z "$out_path" ]; then
  out_path="${summaries_dir}/workbench-full-benchmark-repeats.json"
fi

entries_file="$(mktemp)"
compare_samples_file="$(mktemp)"
step_samples_file="$(mktemp)"
trap 'rm -f "$entries_file" "$compare_samples_file" "$step_samples_file"' EXIT
: >"$entries_file"
: >"$compare_samples_file"
: >"$step_samples_file"

append_compare_samples() {
  local matrix_path="$1"
  local run_tag="$2"
  jq -c \
    --arg run "$run_tag" \
    '.endpoints[]?.compared[]? | {
      run: $run,
      impl: (.impl // null),
      endpoint: (.endpoint // null),
      requestsPerSec: (.requestsPerSec // null),
      p99: (.p99 // null),
      rssKb: (.rssKb // null)
    }' \
    "$matrix_path" >>"$compare_samples_file"
}

append_step_samples() {
  local step_matrix_path="$1"
  local run_tag="$2"
  jq -c \
    --arg run "$run_tag" \
    '.endpoints[]?.compared[]? | {
      run: $run,
      impl: (.impl // null),
      endpoint: (.endpoint // null),
      kneeDetected: (.kneeDetected // false),
      kneeAtTargetRps: (.kneeAtTargetRps // null),
      achievedRatioMin: (.achievedRatioMin // null),
      achievedRatioMax: (.achievedRatioMax // null),
      p99MinMs: (.p99MinMs // null),
      p99MaxMs: (.p99MaxMs // null)
    }' \
    "$step_matrix_path" >>"$step_samples_file"
}

compute_compare_stats() {
  jq -s '
    def p99_to_ms:
      if . == null then null
      elif (type != "string") then null
      elif test("ms$") then (sub("ms$";"") | tonumber?)
      elif test("us$") then ((sub("us$";"") | tonumber?) / 1000)
      elif test("s$") then ((sub("s$";"") | tonumber?) * 1000)
      else (tonumber?)
      end;
    def metric($vals):
      if ($vals | length) == 0 then null else {
        mean: (($vals | add) / ($vals | length)),
        min: ($vals | min),
        max: ($vals | max)
      } end;
    sort_by((.impl // "") + "|" + (.endpoint // ""))
    | group_by((.impl // "") + "|" + (.endpoint // ""))
    | map({
        impl: .[0].impl,
        endpoint: .[0].endpoint,
        samples: length,
        requestsPerSec: metric([.[].requestsPerSec | select(type == "number")]),
        p99Ms: metric([.[].p99 | p99_to_ms | select(type == "number")]),
        rssKb: metric([.[].rssKb | select(type == "number")])
      })
    | sort_by(.endpoint, .impl)
  ' "$compare_samples_file"
}

compute_step_stats() {
  jq -s '
    def metric($vals):
      if ($vals | length) == 0 then null else {
        mean: (($vals | add) / ($vals | length)),
        min: ($vals | min),
        max: ($vals | max)
      } end;
    sort_by((.impl // "") + "|" + (.endpoint // ""))
    | group_by((.impl // "") + "|" + (.endpoint // ""))
    | map({
        impl: .[0].impl,
        endpoint: .[0].endpoint,
        samples: length,
        kneeDetectedCount: ([.[].kneeDetected | select(. == true)] | length),
        kneeDetectedRatio: (([.[].kneeDetected | select(. == true)] | length) / length),
        kneeAtTargetRps: metric([.[].kneeAtTargetRps | select(type == "number")]),
        achievedRatioMin: metric([.[].achievedRatioMin | select(type == "number")]),
        achievedRatioMax: metric([.[].achievedRatioMax | select(type == "number")]),
        p99MinMs: metric([.[].p99MinMs | select(type == "number")]),
        p99MaxMs: metric([.[].p99MaxMs | select(type == "number")])
      })
    | sort_by(.endpoint, .impl)
  ' "$step_samples_file"
}

for run_index in $(seq 1 "$runs"); do
  run_tag="$(printf 'run-%03d' "$run_index")"

  run_summary_path="${runs_dir}/workbench-full-${run_tag}.json"
  run_fixed_runs="${runs_dir}/workbench-benchmark-runs-${run_tag}.json"
  run_step_runs="${runs_dir}/workbench-step-runs-${run_tag}.json"
  run_compare="${runs_dir}/workbench-compare-matrix-${run_tag}.json"
  run_analysis="${runs_dir}/workbench-analysis-${run_tag}.json"
  run_step_matrix="${runs_dir}/workbench-step-matrix-${run_tag}.json"
  run_report="${runs_dir}/workbench-full-report-${run_tag}.md"

  run_cmd=(
    "${suite_dir}/scripts/run_workbench_full_benchmark_suite.sh"
    "${passthrough[@]}"
    --out-runs "$run_summary_path"
    --out-fixed-runs "$run_fixed_runs"
    --out-step-runs "$run_step_runs"
    --out-compare "$run_compare"
    --out-analysis "$run_analysis"
    --out-step-matrix "$run_step_matrix"
    --out-report "$run_report"
  )

  echo "repeat-run: ${run_index}/${runs} (${run_tag})"
  echo "run: ${run_cmd[*]}"
  "${run_cmd[@]}"

  if [ "$dry_run" != "true" ]; then
    append_compare_samples "$run_compare" "$run_tag"
    append_step_samples "$run_step_matrix" "$run_tag"
  fi

  jq -n \
    --arg run "$run_tag" \
    --arg summary "$run_summary_path" \
    --arg fixedRuns "$run_fixed_runs" \
    --arg stepRuns "$run_step_runs" \
    --arg compare "$run_compare" \
    --arg analysis "$run_analysis" \
    --arg stepMatrix "$run_step_matrix" \
    --arg report "$run_report" \
    '{
      run: $run,
      summary: $summary,
      fixedRuns: $fixedRuns,
      stepRuns: $stepRuns,
      compareMatrix: $compare,
      analysis: $analysis,
      stepMatrix: $stepMatrix,
      report: $report
    }' >>"$entries_file"
done

runs_json="$(jq -s '.' "$entries_file")"
compare_stats_json='null'
step_stats_json='null'
if [ "$dry_run" != "true" ]; then
  compare_stats_json="$(compute_compare_stats)"
  step_stats_json="$(compute_step_stats)"
fi

jq -n \
  --arg mode "workbench-full-benchmark-suite-repeats" \
  --arg generatedAt "$(date -u +%FT%TZ)" \
  --arg dryRun "$dry_run" \
  --argjson runCount "$runs" \
  --arg wrapper "${suite_dir}/scripts/run_workbench_full_benchmark_suite_repeats.sh" \
  --arg inner "${suite_dir}/scripts/run_workbench_full_benchmark_suite.sh" \
  --arg passthrough "$(printf '%s\n' "${passthrough[@]}" | jq -R . | jq -s .)" \
  --argjson runs "$runs_json" \
  --argjson compareStats "$compare_stats_json" \
  --argjson stepStats "$step_stats_json" \
  '{
    mode: $mode,
    generatedAt: $generatedAt,
    dryRun: ($dryRun == "true"),
    runCount: $runCount,
    scripts: {
      wrapper: $wrapper,
      inner: $inner
    },
    forwardedArgs: ($passthrough | fromjson),
    runs: $runs,
    compareStats: $compareStats,
    stepStats: $stepStats
  }' >"$out_path"

echo "wrote ${out_path}"
