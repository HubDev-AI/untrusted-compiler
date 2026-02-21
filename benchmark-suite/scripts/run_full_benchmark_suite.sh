#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls sec4,sec4-lasm,node,go,rust,c] [--endpoints ping,decode,users-post,users-get] [--sec-audit path]
          [--include-lasm-mode-compare]
          [--include-lasm-saturation] [--saturation-skip-verify] [--saturation-boost-steps csv]
          [--saturation-project-path path] [--saturation-duration duration] [--saturation-threads n]
          [--saturation-connections n] [--saturation-target-requests n]
          [--saturation-cluster-relay-workers n] [--saturation-cluster-relay-queue n]
          [--saturation-cluster-accept-workers n] [--saturation-cluster-relay-accept-batch-max n]
          [--saturation-cluster-relay-pump-batch-max n] [--saturation-fixed-reuse-port-mode]

Runs fixed-target matrix + step-load matrix and emits a combined markdown report
with step-load signals included.
USAGE
}

dry_run="false"
impls_csv="sec4,sec4-lasm,node,go,rust"
endpoints_csv="ping,decode,users-post,users-get"
sec_audit_path=""
include_lasm_mode_compare="${LASM_INCLUDE_MODE_COMPARE:-false}"
include_lasm_saturation="false"
saturation_skip_verify="false"
saturation_boost_steps_csv="${LASM_CAPACITY_BOOST_STEPS:-2,4,6}"
saturation_project_path=""
saturation_duration=""
saturation_threads=""
saturation_connections=""
saturation_target_requests=""
saturation_cluster_relay_workers=""
saturation_cluster_relay_queue=""
saturation_cluster_accept_workers=""
saturation_cluster_relay_accept_batch_max=""
saturation_cluster_relay_pump_batch_max=""
saturation_fixed_reuse_port_mode="${LASM_CAPACITY_FIXED_REUSE_PORT_MODE:-false}"

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
    --include-lasm-mode-compare)
      include_lasm_mode_compare="true"
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
    --saturation-project-path)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_project_path="$2"
      shift 2
      ;;
    --saturation-project-path=*)
      saturation_project_path="${1#--saturation-project-path=}"
      shift
      ;;
    --saturation-duration)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_duration="$2"
      shift 2
      ;;
    --saturation-duration=*)
      saturation_duration="${1#--saturation-duration=}"
      shift
      ;;
    --saturation-threads)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_threads="$2"
      shift 2
      ;;
    --saturation-threads=*)
      saturation_threads="${1#--saturation-threads=}"
      shift
      ;;
    --saturation-connections)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_connections="$2"
      shift 2
      ;;
    --saturation-connections=*)
      saturation_connections="${1#--saturation-connections=}"
      shift
      ;;
    --saturation-target-requests)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_target_requests="$2"
      shift 2
      ;;
    --saturation-target-requests=*)
      saturation_target_requests="${1#--saturation-target-requests=}"
      shift
      ;;
    --saturation-cluster-relay-workers)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_cluster_relay_workers="$2"
      shift 2
      ;;
    --saturation-cluster-relay-workers=*)
      saturation_cluster_relay_workers="${1#--saturation-cluster-relay-workers=}"
      shift
      ;;
    --saturation-cluster-relay-queue)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_cluster_relay_queue="$2"
      shift 2
      ;;
    --saturation-cluster-relay-queue=*)
      saturation_cluster_relay_queue="${1#--saturation-cluster-relay-queue=}"
      shift
      ;;
    --saturation-cluster-accept-workers)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_cluster_accept_workers="$2"
      shift 2
      ;;
    --saturation-cluster-accept-workers=*)
      saturation_cluster_accept_workers="${1#--saturation-cluster-accept-workers=}"
      shift
      ;;
    --saturation-cluster-relay-accept-batch-max)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_cluster_relay_accept_batch_max="$2"
      shift 2
      ;;
    --saturation-cluster-relay-accept-batch-max=*)
      saturation_cluster_relay_accept_batch_max="${1#--saturation-cluster-relay-accept-batch-max=}"
      shift
      ;;
    --saturation-cluster-relay-pump-batch-max)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      saturation_cluster_relay_pump_batch_max="$2"
      shift 2
      ;;
    --saturation-cluster-relay-pump-batch-max=*)
      saturation_cluster_relay_pump_batch_max="${1#--saturation-cluster-relay-pump-batch-max=}"
      shift
      ;;
    --saturation-fixed-reuse-port-mode)
      saturation_fixed_reuse_port_mode="true"
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
if [ "${include_lasm_mode_compare}" = "true" ] && ! contains_csv_token "${impls_csv}" "sec4-lasm"; then
  echo "--include-lasm-mode-compare requires sec4-lasm in --impls" >&2
  exit 2
fi
if [ "${include_lasm_mode_compare}" != "true" ] && [ "${include_lasm_mode_compare}" != "false" ]; then
  echo "include lasm mode compare must be true or false, got: ${include_lasm_mode_compare}" >&2
  exit 2
fi
if [ "${saturation_fixed_reuse_port_mode}" != "true" ] && [ "${saturation_fixed_reuse_port_mode}" != "false" ]; then
  echo "saturation fixed reuse-port mode must be true or false, got: ${saturation_fixed_reuse_port_mode}" >&2
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
mode_compare_summary_path="${summaries_dir}/sec4-lasm-cluster-mode-compare.json"

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
  if [ -n "${saturation_project_path}" ]; then
    saturation_args+=(--project-path "${saturation_project_path}")
  fi
  if [ -n "${saturation_duration}" ]; then
    saturation_args+=(--duration "${saturation_duration}")
  fi
  if [ -n "${saturation_threads}" ]; then
    saturation_args+=(--threads "${saturation_threads}")
  fi
  if [ -n "${saturation_connections}" ]; then
    saturation_args+=(--connections "${saturation_connections}")
  fi
  if [ -n "${saturation_target_requests}" ]; then
    saturation_args+=(--target-requests "${saturation_target_requests}")
  fi
  if [ -n "${saturation_cluster_relay_workers}" ]; then
    saturation_args+=(--cluster-relay-workers "${saturation_cluster_relay_workers}")
  fi
  if [ -n "${saturation_cluster_relay_queue}" ]; then
    saturation_args+=(--cluster-relay-queue "${saturation_cluster_relay_queue}")
  fi
  if [ -n "${saturation_cluster_accept_workers}" ]; then
    saturation_args+=(--cluster-accept-workers "${saturation_cluster_accept_workers}")
  fi
  if [ -n "${saturation_cluster_relay_accept_batch_max}" ]; then
    saturation_args+=(--cluster-relay-accept-batch-max "${saturation_cluster_relay_accept_batch_max}")
  fi
  if [ -n "${saturation_cluster_relay_pump_batch_max}" ]; then
    saturation_args+=(--cluster-relay-pump-batch-max "${saturation_cluster_relay_pump_batch_max}")
  fi
  if [ "${saturation_fixed_reuse_port_mode}" = "true" ]; then
    saturation_args+=(--fixed-reuse-port-mode)
  fi
  if [ "${saturation_skip_verify}" = "true" ]; then
    saturation_args+=(--skip-verify)
  fi
  if [ "$dry_run" = "true" ]; then
    saturation_args+=(--dry-run)
  fi
  "${root_dir}/scripts/run_lasm_cluster_saturation_boost_bundle.sh" "${saturation_args[@]}"
fi

if [ "${include_lasm_mode_compare}" = "true" ]; then
  echo "phase: lasm mode compare"
  mode_compare_args=(
    --project-path "${saturation_project_path:-examples/lasm-alpha-full}"
  )
  if [ -n "${saturation_duration}" ]; then
    mode_compare_args+=(--duration "${saturation_duration}")
  fi
  if [ -n "${saturation_threads}" ]; then
    mode_compare_args+=(--threads "${saturation_threads}")
  fi
  if [ -n "${saturation_connections}" ]; then
    mode_compare_args+=(--connections "${saturation_connections}")
  fi
  if [ -n "${saturation_target_requests}" ]; then
    mode_compare_args+=(--target-requests "${saturation_target_requests}")
  fi
  if [ -n "${saturation_cluster_relay_workers}" ]; then
    mode_compare_args+=(--cluster-relay-workers "${saturation_cluster_relay_workers}")
  fi
  if [ -n "${saturation_cluster_relay_queue}" ]; then
    mode_compare_args+=(--cluster-relay-queue "${saturation_cluster_relay_queue}")
  fi
  if [ -n "${saturation_cluster_accept_workers}" ]; then
    mode_compare_args+=(--cluster-accept-workers "${saturation_cluster_accept_workers}")
  fi
  if [ -n "${saturation_cluster_relay_accept_batch_max}" ]; then
    mode_compare_args+=(--cluster-relay-accept-batch-max "${saturation_cluster_relay_accept_batch_max}")
  fi
  if [ -n "${saturation_cluster_relay_pump_batch_max}" ]; then
    mode_compare_args+=(--cluster-relay-pump-batch-max "${saturation_cluster_relay_pump_batch_max}")
  fi
  if [ "$dry_run" = "true" ]; then
    mode_compare_args+=(--dry-run)
  fi
  "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" "${mode_compare_args[@]}"
fi

echo "phase: publish combined report"
publish_args=("$matrix_path" "$report_md_path" "${sec_audit_path:-}" "$analysis_path" "$step_matrix_path")
if [ "${include_lasm_saturation}" = "true" ]; then
  publish_args+=("$saturation_summary_path")
fi
if [ "${include_lasm_mode_compare}" = "true" ]; then
  publish_args+=("$mode_compare_summary_path")
fi
if [ "$dry_run" = "true" ]; then
  echo "run: ${root_dir}/scripts/publish_report.sh ${publish_args[*]}"
  echo "run: ${root_dir}/scripts/build_artifact_manifest.sh ${results_dir} ${manifest_path}"
  exit 0
fi

"${root_dir}/scripts/publish_report.sh" "${publish_args[@]}"

"${root_dir}/scripts/build_artifact_manifest.sh" "$results_dir" "$manifest_path"

echo "full benchmark suite report written to ${report_md_path}"
echo "artifact manifest written to ${manifest_path}"
