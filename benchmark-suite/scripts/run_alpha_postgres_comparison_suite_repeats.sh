#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--runs 3] [--dry-run] [--out path] [<alpha-suite args...>]

Runs benchmark-suite/scripts/run_alpha_postgres_comparison_suite.sh repeatedly
under identical arguments, snapshots per-run artifacts to deterministic run-
scoped files, and writes one aggregate run-manifest JSON.

Wrapper options:
  --runs <n>     Number of repeated runs (default: 3)
  --dry-run      Forward dry-run to inner suite
  --out <path>   Aggregate summary output path

All other arguments are forwarded to run_alpha_postgres_comparison_suite.sh.
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

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
summaries_dir="${root_dir}/results/summaries"
runs_dir="${summaries_dir}/alpha-postgres-comparison-suite-runs"
mkdir -p "$summaries_dir" "$runs_dir"

if [ -z "$out_path" ]; then
  out_path="${summaries_dir}/alpha-postgres-comparison-suite-repeats.json"
fi

entries_file="$(mktemp)"
trap 'rm -f "$entries_file"' EXIT
: >"$entries_file"

copy_artifact() {
  local src="$1"
  local dest="$2"
  if [ ! -f "$src" ]; then
    echo "expected artifact missing after run: ${src}" >&2
    exit 2
  fi
  cp -f "$src" "$dest"
}

for run_index in $(seq 1 "$runs"); do
  run_tag="$(printf 'run-%03d' "$run_index")"
  run_summary_path="${runs_dir}/alpha-postgres-comparison-suite-${run_tag}.json"
  run_cmd=(
    "${root_dir}/scripts/run_alpha_postgres_comparison_suite.sh"
    "${passthrough[@]}"
    --out "$run_summary_path"
  )

  echo "repeat-run: ${run_index}/${runs} (${run_tag})"
  echo "run: ${run_cmd[*]}"
  "${run_cmd[@]}"

  if [ "$dry_run" = "true" ]; then
    jq -n \
      --arg run "$run_tag" \
      --arg summary "$run_summary_path" \
      '{run: $run, dryRun: true, summary: $summary}' >>"$entries_file"
    continue
  fi

  base_matrix_src="$(jq -r '.baseline.artifacts.matrix // empty' "$run_summary_path")"
  base_analysis_src="$(jq -r '.baseline.artifacts.analysis // empty' "$run_summary_path")"
  base_step_src="$(jq -r '.baseline.artifacts.stepMatrix // empty' "$run_summary_path")"
  base_report_src="$(jq -r '.baseline.artifacts.report // empty' "$run_summary_path")"
  base_manifest_src="$(jq -r '.baseline.artifacts.manifest // empty' "$run_summary_path")"
  db_matrix_src="$(jq -r '.dbHot.artifacts.matrix // empty' "$run_summary_path")"
  db_analysis_src="$(jq -r '.dbHot.artifacts.analysis // empty' "$run_summary_path")"
  db_step_src="$(jq -r '.dbHot.artifacts.stepMatrix // empty' "$run_summary_path")"
  db_report_src="$(jq -r '.dbHot.artifacts.report // empty' "$run_summary_path")"
  db_manifest_src="$(jq -r '.dbHot.artifacts.manifest // empty' "$run_summary_path")"

  base_matrix_copy="${runs_dir}/compare-matrix-alpha-base-${run_tag}.json"
  base_analysis_copy="${runs_dir}/analysis-alpha-base-${run_tag}.json"
  base_step_copy="${runs_dir}/step-matrix-alpha-base-${run_tag}.json"
  base_report_copy="${runs_dir}/benchmark-report-alpha-base-${run_tag}.md"
  base_manifest_copy="${runs_dir}/artifact-manifest-alpha-base-${run_tag}.json"
  db_matrix_copy="${runs_dir}/compare-matrix-alpha-db-postgres-${run_tag}.json"
  db_analysis_copy="${runs_dir}/analysis-alpha-db-postgres-${run_tag}.json"
  db_step_copy="${runs_dir}/step-matrix-alpha-db-postgres-${run_tag}.json"
  db_report_copy="${runs_dir}/benchmark-report-alpha-db-postgres-${run_tag}.md"
  db_manifest_copy="${runs_dir}/artifact-manifest-alpha-db-postgres-${run_tag}.json"

  copy_artifact "$base_matrix_src" "$base_matrix_copy"
  copy_artifact "$base_analysis_src" "$base_analysis_copy"
  copy_artifact "$base_step_src" "$base_step_copy"
  copy_artifact "$base_report_src" "$base_report_copy"
  copy_artifact "$base_manifest_src" "$base_manifest_copy"
  copy_artifact "$db_matrix_src" "$db_matrix_copy"
  copy_artifact "$db_analysis_src" "$db_analysis_copy"
  copy_artifact "$db_step_src" "$db_step_copy"
  copy_artifact "$db_report_src" "$db_report_copy"
  copy_artifact "$db_manifest_src" "$db_manifest_copy"

  tmp_summary="$(mktemp)"
  jq \
    --arg run "$run_tag" \
    --arg baseMatrix "$base_matrix_copy" \
    --arg baseAnalysis "$base_analysis_copy" \
    --arg baseStep "$base_step_copy" \
    --arg baseReport "$base_report_copy" \
    --arg baseManifest "$base_manifest_copy" \
    --arg dbMatrix "$db_matrix_copy" \
    --arg dbAnalysis "$db_analysis_copy" \
    --arg dbStep "$db_step_copy" \
    --arg dbReport "$db_report_copy" \
    --arg dbManifest "$db_manifest_copy" \
    '.run = $run
     | .baseline.artifacts.matrix = $baseMatrix
     | .baseline.artifacts.analysis = $baseAnalysis
     | .baseline.artifacts.stepMatrix = $baseStep
     | .baseline.artifacts.report = $baseReport
     | .baseline.artifacts.manifest = $baseManifest
     | .dbHot.artifacts.matrix = $dbMatrix
     | .dbHot.artifacts.analysis = $dbAnalysis
     | .dbHot.artifacts.stepMatrix = $dbStep
     | .dbHot.artifacts.report = $dbReport
     | .dbHot.artifacts.manifest = $dbManifest' \
    "$run_summary_path" >"$tmp_summary"
  mv "$tmp_summary" "$run_summary_path"

  jq -n \
    --arg run "$run_tag" \
    --arg summary "$run_summary_path" \
    --arg baseMatrix "$base_matrix_copy" \
    --arg dbMatrix "$db_matrix_copy" \
    '{run: $run, dryRun: false, summary: $summary, baselineMatrix: $baseMatrix, dbHotMatrix: $dbMatrix}' \
    >>"$entries_file"
done

runs_json="$(jq -s '.' "$entries_file")"
jq -n \
  --arg mode "alpha-postgres-comparison-suite-repeats" \
  --arg generatedAt "$(date -u +%FT%TZ)" \
  --arg dryRun "$dry_run" \
  --argjson runCount "$runs" \
  --argjson runs "$runs_json" \
  --arg wrapper "${root_dir}/scripts/run_alpha_postgres_comparison_suite_repeats.sh" \
  --arg inner "${root_dir}/scripts/run_alpha_postgres_comparison_suite.sh" \
  --arg passthrough "$(printf '%s\n' "${passthrough[@]}" | jq -R . | jq -s .)" \
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
    runs: $runs
  }' >"$out_path"

echo "wrote ${out_path}"
