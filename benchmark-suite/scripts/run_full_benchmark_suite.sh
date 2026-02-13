#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls ailang,node,go,rust,c] [--endpoints ping,decode,users-post,users-get] [--sec-audit path]

Runs fixed-target matrix + step-load matrix and emits a combined markdown report
with step-load signals included.
USAGE
}

dry_run="false"
impls_csv="ailang,node,go,rust"
endpoints_csv="ping,decode,users-post,users-get"
sec_audit_path=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
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
matrix_path="${summaries_dir}/compare-matrix.json"
analysis_path="${summaries_dir}/analysis.json"
step_matrix_path="${summaries_dir}/step-matrix.json"
report_md_path="${results_dir}/benchmark-report.md"

if [ -z "$sec_audit_path" ]; then
  candidate="${root_dir}/../baselines/sec-audit/default-secure-prod.hello.json"
  if [ -f "$candidate" ]; then
    sec_audit_path="$candidate"
  fi
fi

echo "phase: fixed-target matrix"
if [ "$dry_run" = "true" ]; then
  "${root_dir}/scripts/run_comparison_matrix.sh" --dry-run --impls "$impls_csv" --endpoints "$endpoints_csv" ${sec_audit_path:+--sec-audit "$sec_audit_path"}
else
  "${root_dir}/scripts/run_comparison_matrix.sh" --impls "$impls_csv" --endpoints "$endpoints_csv" ${sec_audit_path:+--sec-audit "$sec_audit_path"}
fi

echo "phase: step-load matrix"
if [ "$dry_run" = "true" ]; then
  "${root_dir}/scripts/run_step_matrix.sh" --dry-run --impls "$impls_csv" --endpoints "$endpoints_csv"
else
  "${root_dir}/scripts/run_step_matrix.sh" --impls "$impls_csv" --endpoints "$endpoints_csv"
fi

echo "phase: publish combined report"
if [ "$dry_run" = "true" ]; then
  if [ -n "$sec_audit_path" ]; then
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} ${sec_audit_path} ${analysis_path} ${step_matrix_path}"
  else
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} \"\" ${analysis_path} ${step_matrix_path}"
  fi
  exit 0
fi

if [ -n "$sec_audit_path" ]; then
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "$sec_audit_path" "$analysis_path" "$step_matrix_path"
else
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "" "$analysis_path" "$step_matrix_path"
fi

echo "full benchmark suite report written to ${report_md_path}"
