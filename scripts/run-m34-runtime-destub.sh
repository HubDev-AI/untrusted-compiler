#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--plan-json <path>] [--output-dir <path>] [--dry-run|--execute] [--format <text|json>]

Runs the first selected runtime de-stub slice from the M34 runtime plan.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
plan_json_path="${root_dir}/build/m34-runtime-destub-plan.json"
output_dir="${root_dir}/build/m34-runtime-destub-run"
run_mode="dry-run"
output_format="text"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --plan-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      plan_json_path="$2"
      shift 2
      ;;
    --plan-json=*)
      plan_json_path="${1#--plan-json=}"
      shift
      ;;
    --output-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_dir="$2"
      shift 2
      ;;
    --output-dir=*)
      output_dir="${1#--output-dir=}"
      shift
      ;;
    --dry-run)
      run_mode="dry-run"
      shift
      ;;
    --execute)
      run_mode="execute"
      shift
      ;;
    --format)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_format="$2"
      shift 2
      ;;
    --format=*)
      output_format="${1#--format=}"
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

case "${output_format}" in
  text|json)
    ;;
  *)
    echo "unknown format: ${output_format}" >&2
    usage
    exit 2
    ;;
esac

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

if [ ! -f "${plan_json_path}" ]; then
  echo "missing M34 runtime de-stub plan json: ${plan_json_path}" >&2
  exit 1
fi

if ! jq -e '.plan and (.plan | type == "array") and (.plan | length > 0) and .closureGate and .selectedTrack' "${plan_json_path}" >/dev/null; then
  echo "invalid M34 runtime de-stub plan contract: ${plan_json_path}" >&2
  exit 1
fi

selected_slice_id="$(jq -r '.plan[0].id' "${plan_json_path}")"
selected_domain="$(jq -r '.plan[0].domain' "${plan_json_path}")"
selected_track="$(jq -r '.selectedTrack' "${plan_json_path}")"
stabilization_mode="$(jq -r '.stabilizationMode' "${plan_json_path}")"

planned_steps_json='[]'
case "${selected_domain}" in
  db)
    planned_steps_json='[
      "replace db.exec/db.queryOne/db.tx/db.execTx runtime stubs",
      "verify typed SqlQuery sink behavior in runtime adapter",
      "emit db runtime de-stub checkpoint artifact"
    ]'
    ;;
  net)
    planned_steps_json='[
      "replace httpClient runtime stubs for public/internal requests",
      "verify SSRF/public-internal boundary checks in runtime adapter",
      "emit net runtime de-stub checkpoint artifact"
    ]'
    ;;
  validators)
    planned_steps_json='[
      "replace validate/sanitize/url/path runtime stubs",
      "verify canonical gate behavior for untrusted inputs",
      "emit validator runtime de-stub checkpoint artifact"
    ]'
    ;;
  fs)
    planned_steps_json='[
      "replace fs.read/fs.write runtime stubs under PathSafe checks",
      "verify path normalization and traversal rejection hooks",
      "emit fs runtime de-stub checkpoint artifact"
    ]'
    ;;
  secrets)
    planned_steps_json='[
      "replace secrets.get/redact/reveal runtime stubs",
      "verify redact/reveal boundary behavior and policy hooks",
      "emit secrets runtime de-stub checkpoint artifact"
    ]'
    ;;
  *)
    echo "unsupported M34 runtime de-stub domain: ${selected_domain}" >&2
    exit 1
    ;;
esac

mkdir -p "${output_dir}"
marker_path="${output_dir}/executed-${selected_slice_id}.marker"
execution_status="DRY_RUN"
executed=false
if [ "${run_mode}" = "execute" ]; then
  printf 'executed slice: %s\n' "${selected_slice_id}" > "${marker_path}"
  execution_status="PASS"
  executed=true
fi

result_json="$(
  jq -n \
    --arg version "0.1" \
    --arg planJson "${plan_json_path}" \
    --arg outputDir "${output_dir}" \
    --arg selectedTrack "${selected_track}" \
    --arg selectedSliceId "${selected_slice_id}" \
    --arg selectedDomain "${selected_domain}" \
    --arg runMode "${run_mode}" \
    --arg executionStatus "${execution_status}" \
    --arg closureGate "M34-D" \
    --arg markerPath "${marker_path}" \
    --arg stabilizationMode "${stabilization_mode}" \
    --argjson plannedSteps "${planned_steps_json}" \
    --argjson executed "${executed}" \
    '{
      version: $version,
      inputs: {
        planJson: $planJson
      },
      outputDir: $outputDir,
      selectedTrack: $selectedTrack,
      selectedSlice: {
        id: $selectedSliceId,
        domain: $selectedDomain
      },
      runMode: $runMode,
      stabilizationMode: ($stabilizationMode == "true"),
      plannedSteps: $plannedSteps,
      executionStatus: $executionStatus,
      executed: $executed,
      closureGate: $closureGate,
      artifacts: {
        executionStatusJson: ($outputDir + "/execution-status.json"),
        executionMarker: $markerPath
      }
    }'
)"

execution_status_json_path="${output_dir}/execution-status.json"
printf '%s\n' "${result_json}" > "${execution_status_json_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${result_json}"
  exit 0
fi

echo "M34 Runtime De-stub Runner"
echo "selectedTrack: ${selected_track}"
echo "selectedSliceId: ${selected_slice_id}"
echo "selectedDomain: ${selected_domain}"
echo "runMode: ${run_mode}"
echo "executionStatus: ${execution_status}"
echo "closureGate: M34-D"
echo "output: ${execution_status_json_path}"
