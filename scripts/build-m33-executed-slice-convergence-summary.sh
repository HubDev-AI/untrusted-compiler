#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--plan-json <path>] [--runtime-execution-json <path>] [--output <path>] [--format <markdown|json>]

Builds a deterministic convergence summary for the first executed M33 runtime de-stub slice.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
plan_json_path="${root_dir}/build/m33-runtime-destub-plan.json"
runtime_execution_json_path="${root_dir}/build/m33-runtime-destub-run/execution-status.json"
output_path="${root_dir}/build/m33-executed-slice-convergence-summary.md"
output_format="markdown"

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
    --runtime-execution-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      runtime_execution_json_path="$2"
      shift 2
      ;;
    --runtime-execution-json=*)
      runtime_execution_json_path="${1#--runtime-execution-json=}"
      shift
      ;;
    --output)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_path="$2"
      shift 2
      ;;
    --output=*)
      output_path="${1#--output=}"
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
  markdown|json)
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
  echo "missing M33 runtime de-stub plan json: ${plan_json_path}" >&2
  exit 1
fi
if [ ! -f "${runtime_execution_json_path}" ]; then
  echo "missing M33 runtime execution json: ${runtime_execution_json_path}" >&2
  exit 1
fi

if ! jq -e '.selectedTrack and .closureGate and .plan and (.plan | type == "array") and (.plan | length > 0) and .plan[0].id' "${plan_json_path}" >/dev/null; then
  echo "invalid M33 runtime de-stub plan contract: ${plan_json_path}" >&2
  exit 1
fi
if ! jq -e '.selectedTrack and .selectedSlice and .selectedSlice.id and .executionStatus and .closureGate' "${runtime_execution_json_path}" >/dev/null; then
  echo "invalid M33 runtime execution contract: ${runtime_execution_json_path}" >&2
  exit 1
fi

plan_track="$(jq -r '.selectedTrack' "${plan_json_path}")"
plan_slice_id="$(jq -r '.plan[0].id' "${plan_json_path}")"
plan_closure_gate="$(jq -r '.closureGate' "${plan_json_path}")"

runtime_track="$(jq -r '.selectedTrack' "${runtime_execution_json_path}")"
runtime_slice_id="$(jq -r '.selectedSlice.id' "${runtime_execution_json_path}")"
runtime_status="$(jq -r '.executionStatus' "${runtime_execution_json_path}")"
runtime_closure_gate="$(jq -r '.closureGate' "${runtime_execution_json_path}")"

if [ "${runtime_track}" != "${plan_track}" ]; then
  echo "runtime execution selectedTrack does not match plan track: ${runtime_track} != ${plan_track}" >&2
  exit 1
fi
if [ "${runtime_slice_id}" != "${plan_slice_id}" ]; then
  echo "runtime execution selectedSlice.id does not match plan slice id: ${runtime_slice_id} != ${plan_slice_id}" >&2
  exit 1
fi

execution_pass="false"
if [ "${runtime_status}" = "PASS" ] || [ "${runtime_status}" = "DRY_RUN" ]; then
  execution_pass="true"
fi

overall="PENDING"
next_action="Resolve M33 runtime de-stub execution failures before proceeding."
if [ "${execution_pass}" = "true" ]; then
  overall="PASS"
  next_action="Proceed to M33 transition handoff packet preparation."
fi

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg planJson "${plan_json_path}" \
    --arg runtimeExecutionJson "${runtime_execution_json_path}" \
    --arg selectedTrack "${plan_track}" \
    --arg selectedSliceId "${plan_slice_id}" \
    --arg planClosureGate "${plan_closure_gate}" \
    --arg runtimeClosureGate "${runtime_closure_gate}" \
    --arg runtimeStatus "${runtime_status}" \
    --arg overall "${overall}" \
    --arg nextAction "${next_action}" \
    --arg convergenceClosureGate "M33-E" \
    --argjson executionPass "$( [ "${execution_pass}" = "true" ] && echo true || echo false )" \
    '{
      version: $version,
      inputs: {
        planJson: $planJson,
        runtimeExecutionJson: $runtimeExecutionJson
      },
      selectedTrack: $selectedTrack,
      selectedSliceId: $selectedSliceId,
      planClosureGate: $planClosureGate,
      runtimeClosureGate: $runtimeClosureGate,
      runtimeStatus: $runtimeStatus,
      executionPass: $executionPass,
      overall: $overall,
      nextAction: $nextAction,
      convergenceClosureGate: $convergenceClosureGate
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"
{
  echo "# M33 Executed Slice Convergence Summary"
  echo
  printf -- "- planJson: \`%s\`\n" "${plan_json_path}"
  printf -- "- runtimeExecutionJson: \`%s\`\n" "${runtime_execution_json_path}"
  printf -- "- selectedTrack: \`%s\`\n" "${plan_track}"
  printf -- "- selectedSliceId: \`%s\`\n" "${plan_slice_id}"
  printf -- "- planClosureGate: \`%s\`\n" "${plan_closure_gate}"
  printf -- "- runtimeClosureGate: \`%s\`\n" "${runtime_closure_gate}"
  printf -- "- runtimeStatus: \`%s\`\n" "${runtime_status}"
  printf -- "- executionPass: \`%s\`\n" "${execution_pass}"
  printf -- "- overall: \`%s\`\n" "${overall}"
  printf -- "- convergenceClosureGate: \`%s\`\n" "M33-E"
  echo
  echo "## Next Action"
  printf -- "- %s\n" "${next_action}"
} > "${output_path}"

echo "m33 executed-slice convergence summary generated: ${output_path}"
