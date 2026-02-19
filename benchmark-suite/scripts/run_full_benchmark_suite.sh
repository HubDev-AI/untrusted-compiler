#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls sec4,sec4-lasm,node,go,rust,c] [--endpoints ping,decode,users-post,users-get] [--sec-audit path] [--include-lasm-saturation] [--saturation-skip-verify] [--saturation-boost-steps csv]

Runs fixed-target matrix + step-load matrix and emits a combined markdown report
with step-load signals included.
USAGE
}

dry_run="false"
impls_csv="sec4,sec4-lasm,node,go,rust"
endpoints_csv="ping,decode,users-post,users-get"
sec_audit_path=""
include_lasm_saturation="false"
saturation_skip_verify="false"
saturation_boost_steps_csv="${LASM_CAPACITY_BOOST_STEPS:-2,4,6}"

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
    --include-lasm-saturation)
      include_lasm_saturation="true"
      shift
      ;;
    --saturation-skip-verify)
      saturation_skip_verify="true"
      shift
      ;;
    --saturation-boost-steps)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_boost_steps_csv="$2"
      shift 2
      ;;
    --saturation-boost-steps=*)
      saturation_boost_steps_csv="${1#--saturation-boost-steps=}"
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

contains_csv_token() {
  local csv="$1"
  local token="$2"
  IFS=',' read -r -a _items <<< "${csv}"
  for _item in "${_items[@]}"; do
    local trimmed="${_item// /}"
    if [ "${trimmed}" = "${token}" ]; then
      return 0
    fi
  done
  return 1
}

if [ "${include_lasm_saturation}" = "true" ] && ! contains_csv_token "${impls_csv}" "sec4-lasm"; then
  echo "--include-lasm-saturation requires sec4-lasm in --impls" >&2
  exit 2
fi

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
summaries_dir="${root_dir}/results/summaries"
results_dir="${root_dir}/results"
matrix_path="${summaries_dir}/compare-matrix.json"
analysis_path="${summaries_dir}/analysis.json"
step_matrix_path="${summaries_dir}/step-matrix.json"
report_md_path="${results_dir}/benchmark-report.md"
manifest_path="${results_dir}/artifact-manifest.json"
saturation_summary_path="${summaries_dir}/sec4-lasm-cluster-saturation-boost-summary.md"

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

if [ "${include_lasm_saturation}" = "true" ]; then
  echo "phase: lasm saturation tuning bundle"
  saturation_args=(
    --boost-steps "$saturation_boost_steps_csv"
    --summary-out "$saturation_summary_path"
  )
  if [ "${saturation_skip_verify}" = "true" ]; then
    saturation_args+=(--skip-verify)
  fi
  if [ "$dry_run" = "true" ]; then
    saturation_args+=(--dry-run)
  fi
  "${root_dir}/scripts/run_lasm_cluster_saturation_boost_bundle.sh" "${saturation_args[@]}"
fi

echo "phase: publish combined report"
if [ "$dry_run" = "true" ]; then
  if [ "${include_lasm_saturation}" = "true" ]; then
    if [ -n "$sec_audit_path" ]; then
      echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} ${sec_audit_path} ${analysis_path} ${step_matrix_path} ${saturation_summary_path}"
    else
      echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} \"\" ${analysis_path} ${step_matrix_path} ${saturation_summary_path}"
    fi
  else
    if [ -n "$sec_audit_path" ]; then
      echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} ${sec_audit_path} ${analysis_path} ${step_matrix_path}"
    else
      echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} \"\" ${analysis_path} ${step_matrix_path}"
    fi
  fi
  echo "run: ${root_dir}/scripts/build_artifact_manifest.sh ${results_dir} ${manifest_path}"
  exit 0
fi

if [ "${include_lasm_saturation}" = "true" ]; then
  if [ -n "$sec_audit_path" ]; then
    "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "$sec_audit_path" "$analysis_path" "$step_matrix_path" "$saturation_summary_path"
  else
    "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "" "$analysis_path" "$step_matrix_path" "$saturation_summary_path"
  fi
else
  if [ -n "$sec_audit_path" ]; then
    "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "$sec_audit_path" "$analysis_path" "$step_matrix_path"
  else
    "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "" "$analysis_path" "$step_matrix_path"
  fi
fi

"${root_dir}/scripts/build_artifact_manifest.sh" "$results_dir" "$manifest_path"

echo "full benchmark suite report written to ${report_md_path}"
echo "artifact manifest written to ${manifest_path}"
