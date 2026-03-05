#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--matrix path] [--impls sec4,sec4-lasm,node,go,rust]
          [--endpoints wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list]
          [--lasm-db-adapter sqlite|postgres] [--lasm-db-base path] [--lasm-postgres-dsn-file path]
          [--port <n>] [--out-runs path] [--out-fixed-runs path] [--out-step-runs path]
          [--out-compare path] [--out-analysis path] [--out-step-matrix path]
          [--out-report path] [--out-report-html path]

Runs workbench fixed-target matrix + workbench step-load matrix and republishes one
combined markdown report containing step-load signals.
USAGE
}

dry_run="false"
matrix_path=""
impls_csv="sec4,sec4-lasm,node,go,rust"
endpoints_csv="wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list"
bench_port="${BENCH_WORKBENCH_PORT:-18093}"
lasm_db_adapter="${BENCH_WORKBENCH_LASM_DB_ADAPTER:-sqlite}"
lasm_db_base="${BENCH_WORKBENCH_LASM_DB_BASE:-}"
lasm_postgres_dsn_file="${BENCH_WORKBENCH_LASM_POSTGRES_DSN_FILE:-}"
out_runs=""
out_fixed_runs=""
out_step_runs=""
out_compare=""
out_analysis=""
out_step_matrix=""
out_report=""
out_report_html=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    --matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      impls_csv="$2"
      shift 2
      ;;
    --impls=*)
      impls_csv="${1#--impls=}"
      shift
      ;;
    --endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      endpoints_csv="$2"
      shift 2
      ;;
    --endpoints=*)
      endpoints_csv="${1#--endpoints=}"
      shift
      ;;
    --port)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      bench_port="$2"
      shift 2
      ;;
    --port=*)
      bench_port="${1#--port=}"
      shift
      ;;
    --lasm-db-adapter)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_adapter="$2"
      shift 2
      ;;
    --lasm-db-adapter=*)
      lasm_db_adapter="${1#--lasm-db-adapter=}"
      shift
      ;;
    --lasm-db-base)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_base="$2"
      shift 2
      ;;
    --lasm-db-base=*)
      lasm_db_base="${1#--lasm-db-base=}"
      shift
      ;;
    --lasm-postgres-dsn-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_postgres_dsn_file="$2"
      shift 2
      ;;
    --lasm-postgres-dsn-file=*)
      lasm_postgres_dsn_file="${1#--lasm-postgres-dsn-file=}"
      shift
      ;;
    --out-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_runs="$2"
      shift 2
      ;;
    --out-runs=*)
      out_runs="${1#--out-runs=}"
      shift
      ;;
    --out-fixed-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_fixed_runs="$2"
      shift 2
      ;;
    --out-fixed-runs=*)
      out_fixed_runs="${1#--out-fixed-runs=}"
      shift
      ;;
    --out-step-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_step_runs="$2"
      shift 2
      ;;
    --out-step-runs=*)
      out_step_runs="${1#--out-step-runs=}"
      shift
      ;;
    --out-compare)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_compare="$2"
      shift 2
      ;;
    --out-compare=*)
      out_compare="${1#--out-compare=}"
      shift
      ;;
    --out-analysis)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_analysis="$2"
      shift 2
      ;;
    --out-analysis=*)
      out_analysis="${1#--out-analysis=}"
      shift
      ;;
    --out-step-matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_step_matrix="$2"
      shift 2
      ;;
    --out-step-matrix=*)
      out_step_matrix="${1#--out-step-matrix=}"
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
    --out-report-html)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_report_html="$2"
      shift 2
      ;;
    --out-report-html=*)
      out_report_html="${1#--out-report-html=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"

if [ -z "$matrix_path" ]; then
  matrix_path="${suite_dir}/workbench/matrix.backends.json"
fi
if [ -z "$out_runs" ]; then
  out_runs="${suite_dir}/results/summaries/workbench-full-runs.json"
fi
if [ -z "$out_fixed_runs" ]; then
  out_fixed_runs="${suite_dir}/results/summaries/workbench-benchmark-runs.json"
fi
if [ -z "$out_step_runs" ]; then
  out_step_runs="${suite_dir}/results/summaries/workbench-step-runs.json"
fi
if [ -z "$out_compare" ]; then
  out_compare="${suite_dir}/results/summaries/workbench-benchmark-compare-matrix.json"
fi
if [ -z "$out_analysis" ]; then
  out_analysis="${suite_dir}/results/summaries/workbench-benchmark-analysis.json"
fi
if [ -z "$out_step_matrix" ]; then
  out_step_matrix="${suite_dir}/results/summaries/workbench-step-matrix.json"
fi
if [ -z "$out_report" ]; then
  out_report="${suite_dir}/results/workbench-full-benchmark-report.md"
fi
if [ -z "$out_report_html" ]; then
  out_report_html="${suite_dir}/results/workbench-full-benchmark-report.html"
fi

mkdir -p "$(dirname "$out_runs")" "$(dirname "$out_report")" "$(dirname "$out_report_html")"

bench_cmd=(
  "${suite_dir}/scripts/run_workbench_benchmark_matrix.sh"
  --matrix "$matrix_path"
  --impls "$impls_csv"
  --endpoints "$endpoints_csv"
  --port "$bench_port"
  --out-runs "$out_fixed_runs"
  --out-compare "$out_compare"
  --out-analysis "$out_analysis"
  --out-report "$out_report"
  --out-report-html "$out_report_html"
)

step_cmd=(
  "${suite_dir}/scripts/run_workbench_step_matrix.sh"
  --matrix "$matrix_path"
  --impls "$impls_csv"
  --endpoints "$endpoints_csv"
  --port "$bench_port"
  --out-runs "$out_step_runs"
  --out-step-matrix "$out_step_matrix"
)

if [ -n "$lasm_db_adapter" ]; then
  bench_cmd+=(--lasm-db-adapter "$lasm_db_adapter")
  step_cmd+=(--lasm-db-adapter "$lasm_db_adapter")
fi
if [ -n "$lasm_db_base" ]; then
  bench_cmd+=(--lasm-db-base "$lasm_db_base")
  step_cmd+=(--lasm-db-base "$lasm_db_base")
fi
if [ -n "$lasm_postgres_dsn_file" ]; then
  bench_cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
  step_cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
fi
if [ "$dry_run" = "true" ]; then
  bench_cmd=("${bench_cmd[@]:0:1}" --dry-run "${bench_cmd[@]:1}")
  step_cmd=("${step_cmd[@]:0:1}" --dry-run "${step_cmd[@]:1}")
fi

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

"${bench_cmd[@]}"
"${step_cmd[@]}"

if [ "$dry_run" = "true" ]; then
  echo "run: ${suite_dir}/scripts/publish_report.sh ${out_compare} ${out_report} '' ${out_analysis} ${out_step_matrix}"
  exit 0
fi

"${suite_dir}/scripts/publish_report.sh" "$out_compare" "$out_report" "" "$out_analysis" "$out_step_matrix"

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
  --arg version "0.1" \
  --arg startedAt "$started_at" \
  --arg finishedAt "$finished_at" \
  --arg matrixPath "$matrix_path" \
  --arg impls "$impls_csv" \
  --arg endpoints "$endpoints_csv" \
  --argjson port "$bench_port" \
  --arg fixedRunsPath "$out_fixed_runs" \
  --arg stepRunsPath "$out_step_runs" \
  --arg comparePath "$out_compare" \
  --arg analysisPath "$out_analysis" \
  --arg stepMatrixPath "$out_step_matrix" \
  --arg reportPath "$out_report" \
  --arg reportHtmlPath "$out_report_html" \
  '{
    version: $version,
    startedAt: $startedAt,
    finishedAt: $finishedAt,
    matrixPath: $matrixPath,
    impls: ($impls | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
    endpoints: ($endpoints | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
    port: $port,
    artifacts: {
      fixedRuns: $fixedRunsPath,
      stepRuns: $stepRunsPath,
      compareMatrix: $comparePath,
      analysis: $analysisPath,
      stepMatrix: $stepMatrixPath,
      report: $reportPath,
      reportHtml: $reportHtmlPath
    }
  }' >"$out_runs"

echo "wrote workbench full run summary: $out_runs"
echo "wrote workbench full report: $out_report"
echo "wrote workbench full html report: $out_report_html"
