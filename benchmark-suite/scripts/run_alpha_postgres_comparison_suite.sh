#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run]
          [--base-impls sec4,sec4-lasm,node,go,rust]
          [--base-endpoints ping,decode,users-post,users-get]
          [--db-impls sec4-lasm]
          [--db-endpoints db-hot-write,db-hot-write-tx,db-hot-query-one,db-records]
          [--lasm-db-postgres-dsn-file path]
          [--sec-audit path]
          [--out path]

Runs two deterministic suites:
1) Cross-language baseline endpoints with LASM in postgres mode.
2) LASM postgres DB hot-path endpoints.

Each suite snapshots matrix/analysis/step/report/manifest artifacts to stable
suffix paths and writes one summary JSON.
USAGE
}

dry_run="false"
base_impls_csv="sec4,sec4-lasm,node,go,rust"
base_endpoints_csv="ping,decode,users-post,users-get"
db_impls_csv="sec4-lasm"
db_endpoints_csv="db-hot-write,db-hot-write-tx,db-hot-query-one,db-records"
lasm_db_postgres_dsn_file="${BENCH_LASM_DB_POSTGRES_DSN_FILE:-}"
sec_audit_path=""
out_path=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    --base-impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      base_impls_csv="$2"
      shift 2
      ;;
    --base-impls=*)
      base_impls_csv="${1#--base-impls=}"
      shift
      ;;
    --base-endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      base_endpoints_csv="$2"
      shift 2
      ;;
    --base-endpoints=*)
      base_endpoints_csv="${1#--base-endpoints=}"
      shift
      ;;
    --db-impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_impls_csv="$2"
      shift 2
      ;;
    --db-impls=*)
      db_impls_csv="${1#--db-impls=}"
      shift
      ;;
    --db-endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_endpoints_csv="$2"
      shift 2
      ;;
    --db-endpoints=*)
      db_endpoints_csv="${1#--db-endpoints=}"
      shift
      ;;
    --lasm-db-postgres-dsn-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_postgres_dsn_file="$2"
      shift 2
      ;;
    --lasm-db-postgres-dsn-file=*)
      lasm_db_postgres_dsn_file="${1#--lasm-db-postgres-dsn-file=}"
      shift
      ;;
    --sec-audit)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      sec_audit_path="$2"
      shift 2
      ;;
    --sec-audit=*)
      sec_audit_path="${1#--sec-audit=}"
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

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
summaries_dir="${root_dir}/results/summaries"
results_dir="${root_dir}/results"
mkdir -p "$summaries_dir" "$results_dir"

if [ -z "$out_path" ]; then
  out_path="${summaries_dir}/alpha-postgres-comparison-suite.json"
fi

if [ -z "$sec_audit_path" ]; then
  candidate="${root_dir}/../baselines/sec-audit/default-secure-prod.hello.json"
  if [ -f "$candidate" ]; then
    sec_audit_path="$candidate"
  fi
fi

if [ -n "$lasm_db_postgres_dsn_file" ] && [ ! -f "$lasm_db_postgres_dsn_file" ]; then
  echo "lasm postgres dsn file not found: ${lasm_db_postgres_dsn_file}" >&2
  exit 2
fi

if [ -z "$lasm_db_postgres_dsn_file" ] && [ -z "${SEC4_RT_LASM_DB_POSTGRES_DSN:-}" ]; then
  echo "postgres suite requires --lasm-db-postgres-dsn-file or SEC4_RT_LASM_DB_POSTGRES_DSN" >&2
  exit 2
fi

base_run_cmd=(
  "${root_dir}/scripts/run_full_benchmark_suite.sh"
  --impls "$base_impls_csv"
  --endpoints "$base_endpoints_csv"
  --lasm-db-adapter postgres
)
db_run_cmd=(
  "${root_dir}/scripts/run_full_benchmark_suite.sh"
  --impls "$db_impls_csv"
  --endpoints "$db_endpoints_csv"
  --lasm-db-adapter postgres
)
if [ -n "$lasm_db_postgres_dsn_file" ]; then
  base_run_cmd+=(--lasm-db-postgres-dsn-file "$lasm_db_postgres_dsn_file")
  db_run_cmd+=(--lasm-db-postgres-dsn-file "$lasm_db_postgres_dsn_file")
fi
if [ -n "$sec_audit_path" ]; then
  base_run_cmd+=(--sec-audit "$sec_audit_path")
  db_run_cmd+=(--sec-audit "$sec_audit_path")
fi

base_matrix_snapshot="${summaries_dir}/compare-matrix-alpha-base.json"
base_analysis_snapshot="${summaries_dir}/analysis-alpha-base.json"
base_step_snapshot="${summaries_dir}/step-matrix-alpha-base.json"
base_report_snapshot="${results_dir}/benchmark-report-alpha-base.md"
base_manifest_snapshot="${results_dir}/artifact-manifest-alpha-base.json"

db_matrix_snapshot="${summaries_dir}/compare-matrix-alpha-db-postgres.json"
db_analysis_snapshot="${summaries_dir}/analysis-alpha-db-postgres.json"
db_step_snapshot="${summaries_dir}/step-matrix-alpha-db-postgres.json"
db_report_snapshot="${results_dir}/benchmark-report-alpha-db-postgres.md"
db_manifest_snapshot="${results_dir}/artifact-manifest-alpha-db-postgres.json"

snapshot_artifacts() {
  local matrix_path="$1"
  local analysis_path="$2"
  local step_path="$3"
  local report_path="$4"
  local manifest_path="$5"

  cp -f "${summaries_dir}/compare-matrix.json" "$matrix_path"
  cp -f "${summaries_dir}/analysis.json" "$analysis_path"
  cp -f "${summaries_dir}/step-matrix.json" "$step_path"
  cp -f "${results_dir}/benchmark-report.md" "$report_path"
  cp -f "${results_dir}/artifact-manifest.json" "$manifest_path"
}

echo "phase: alpha postgres comparison suite (baseline)"
echo "run: ${base_run_cmd[*]}"
if [ "$dry_run" = "true" ]; then
  "${base_run_cmd[@]}" --dry-run
  echo "snapshot: compare-matrix.json -> ${base_matrix_snapshot}"
  echo "snapshot: analysis.json -> ${base_analysis_snapshot}"
  echo "snapshot: step-matrix.json -> ${base_step_snapshot}"
  echo "snapshot: benchmark-report.md -> ${base_report_snapshot}"
  echo "snapshot: artifact-manifest.json -> ${base_manifest_snapshot}"
else
  "${base_run_cmd[@]}"
  snapshot_artifacts \
    "$base_matrix_snapshot" \
    "$base_analysis_snapshot" \
    "$base_step_snapshot" \
    "$base_report_snapshot" \
    "$base_manifest_snapshot"
fi

echo "phase: alpha postgres comparison suite (db-hot)"
echo "run: ${db_run_cmd[*]}"
if [ "$dry_run" = "true" ]; then
  "${db_run_cmd[@]}" --dry-run
  echo "snapshot: compare-matrix.json -> ${db_matrix_snapshot}"
  echo "snapshot: analysis.json -> ${db_analysis_snapshot}"
  echo "snapshot: step-matrix.json -> ${db_step_snapshot}"
  echo "snapshot: benchmark-report.md -> ${db_report_snapshot}"
  echo "snapshot: artifact-manifest.json -> ${db_manifest_snapshot}"
  exit 0
fi

"${db_run_cmd[@]}"
snapshot_artifacts \
  "$db_matrix_snapshot" \
  "$db_analysis_snapshot" \
  "$db_step_snapshot" \
  "$db_report_snapshot" \
  "$db_manifest_snapshot"

jq -n \
  --arg mode "alpha-postgres-comparison-suite" \
  --arg generatedAt "$(date -u +%FT%TZ)" \
  --arg baseImpls "$base_impls_csv" \
  --arg baseEndpoints "$base_endpoints_csv" \
  --arg dbImpls "$db_impls_csv" \
  --arg dbEndpoints "$db_endpoints_csv" \
  --arg baseMatrix "$base_matrix_snapshot" \
  --arg baseAnalysis "$base_analysis_snapshot" \
  --arg baseStep "$base_step_snapshot" \
  --arg baseReport "$base_report_snapshot" \
  --arg baseManifest "$base_manifest_snapshot" \
  --arg dbMatrix "$db_matrix_snapshot" \
  --arg dbAnalysis "$db_analysis_snapshot" \
  --arg dbStep "$db_step_snapshot" \
  --arg dbReport "$db_report_snapshot" \
  --arg dbManifest "$db_manifest_snapshot" \
  '{
    mode: $mode,
    generatedAt: $generatedAt,
    baseline: {
      impls: ($baseImpls | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
      endpoints: ($baseEndpoints | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
      artifacts: {
        matrix: $baseMatrix,
        analysis: $baseAnalysis,
        stepMatrix: $baseStep,
        report: $baseReport,
        manifest: $baseManifest
      }
    },
    dbHot: {
      impls: ($dbImpls | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
      endpoints: ($dbEndpoints | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
      artifacts: {
        matrix: $dbMatrix,
        analysis: $dbAnalysis,
        stepMatrix: $dbStep,
        report: $dbReport,
        manifest: $dbManifest
      }
    }
  }' > "$out_path"

echo "wrote ${out_path}"
