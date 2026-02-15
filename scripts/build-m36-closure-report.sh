#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--closure-json <path>] [--kickoff-json <path>] [--matrix-json <path>] [--plan-json <path>] [--runtime-execution-json <path>] [--convergence-json <path>] [--packet-json <path>] [--output <path>] [--format <markdown|json>]

Builds a final M36 closure report from strict closure gates and M36 runtime artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
closure_json_path=""
kickoff_json_path="${root_dir}/build/m36-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m36-priority-matrix.json"
plan_json_path="${root_dir}/build/m36-runtime-destub-plan.json"
runtime_execution_json_path="${root_dir}/build/m36-runtime-destub-run/execution-status.json"
convergence_json_path="${root_dir}/build/m36-transition-handoff/convergence.json"
packet_json_path="${root_dir}/build/m36-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m36-closure-report.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      closure_json_path="$2"
      shift 2
      ;;
    --closure-json=*)
      closure_json_path="${1#--closure-json=}"
      shift
      ;;
    --kickoff-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      kickoff_json_path="$2"
      shift 2
      ;;
    --kickoff-json=*)
      kickoff_json_path="${1#--kickoff-json=}"
      shift
      ;;
    --matrix-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      matrix_json_path="$2"
      shift 2
      ;;
    --matrix-json=*)
      matrix_json_path="${1#--matrix-json=}"
      shift
      ;;
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
    --convergence-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      convergence_json_path="$2"
      shift 2
      ;;
    --convergence-json=*)
      convergence_json_path="${1#--convergence-json=}"
      shift
      ;;
    --packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      packet_json_path="$2"
      shift 2
      ;;
    --packet-json=*)
      packet_json_path="${1#--packet-json=}"
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

closure_json=""
closure_source=""
if [ -n "${closure_json_path}" ]; then
  if [ ! -f "${closure_json_path}" ]; then
    echo "missing closure json file: ${closure_json_path}" >&2
    exit 1
  fi
  closure_json="$(cat "${closure_json_path}")"
  closure_source="${closure_json_path}"
else
  closure_json="$("${root_dir}/scripts/check-milestone-closure.sh" --format json)"
  closure_source="live:check-milestone-closure"
fi

if ! printf '%s\n' "${closure_json}" | jq -e '.overall and (.gates | type == "array")' >/dev/null; then
  echo "invalid closure json contract" >&2
  exit 1
fi

if [ ! -f "${kickoff_json_path}" ]; then
  echo "missing M36 kickoff brief json: ${kickoff_json_path}" >&2
  exit 1
fi
if [ ! -f "${matrix_json_path}" ]; then
  echo "missing M36 priority matrix json: ${matrix_json_path}" >&2
  exit 1
fi
if [ ! -f "${plan_json_path}" ]; then
  echo "missing M36 runtime de-stub plan json: ${plan_json_path}" >&2
  exit 1
fi
if [ ! -f "${runtime_execution_json_path}" ]; then
  echo "missing M36 runtime execution json: ${runtime_execution_json_path}" >&2
  exit 1
fi

if ! jq -e '.m35Overall and .convergenceOverall and .primaryFocus and (.pendingGates | type == "array")' "${kickoff_json_path}" >/dev/null; then
  echo "invalid M36 kickoff brief json contract: ${kickoff_json_path}" >&2
  exit 1
fi
if ! jq -e '.tracks and (.tracks | type == "array") and (.tracks | length > 0)' "${matrix_json_path}" >/dev/null; then
  echo "invalid M36 priority matrix contract: ${matrix_json_path}" >&2
  exit 1
fi
if ! jq -e '.selectedTrack and .plan and (.plan | type == "array") and (.plan | length > 0) and .plan[0].id and .closureGate' "${plan_json_path}" >/dev/null; then
  echo "invalid M36 runtime de-stub plan contract: ${plan_json_path}" >&2
  exit 1
fi
if ! jq -e '.selectedTrack and .selectedSlice and .selectedSlice.id and .executionStatus and .closureGate' "${runtime_execution_json_path}" >/dev/null; then
  echo "invalid M36 runtime execution contract: ${runtime_execution_json_path}" >&2
  exit 1
fi

if [ ! -f "${packet_json_path}" ]; then
  packet_output_dir="$(dirname "${packet_json_path}")"
  mkdir -p "${packet_output_dir}"
  packet_generated_path="${packet_output_dir}/handoff-packet.json"

  generate_packet_cmd=(
    "${root_dir}/scripts/build-m36-transition-handoff-packet.sh"
    --kickoff-json "${kickoff_json_path}"
    --matrix-json "${matrix_json_path}"
    --plan-json "${plan_json_path}"
    --runtime-execution-json "${runtime_execution_json_path}"
    --output-dir "${packet_output_dir}"
    --format json
  )
  if [ -f "${convergence_json_path}" ]; then
    generate_packet_cmd+=(--convergence-json "${convergence_json_path}")
  fi
  "${generate_packet_cmd[@]}" >/dev/null

  if [ "${packet_generated_path}" != "${packet_json_path}" ]; then
    cp "${packet_generated_path}" "${packet_json_path}"
  fi
  if [ ! -f "${convergence_json_path}" ] && [ -f "${packet_output_dir}/convergence.json" ]; then
    convergence_json_path="${packet_output_dir}/convergence.json"
  fi
fi

if [ ! -f "${packet_json_path}" ]; then
  echo "missing M36 transition packet json: ${packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.kickoffPrimaryFocus and .summary.planSelectedTrack and .summary.planSelectedSliceId and .summary.runtimeStatus and .summary.convergenceOverall and .artifacts and .artifacts.kickoff and .artifacts.priorityMatrix and .artifacts.runtimePlan and .artifacts.runtimeExecution and .artifacts.convergence and .closureGate' "${packet_json_path}" >/dev/null; then
  echo "invalid M36 transition packet json contract: ${packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${convergence_json_path}" ]; then
  echo "missing M36 executed-slice convergence summary json: ${convergence_json_path}" >&2
  exit 1
fi
if ! jq -e '.overall and .selectedTrack and .selectedSliceId and .convergenceClosureGate' "${convergence_json_path}" >/dev/null; then
  echo "invalid M36 executed-slice convergence summary contract: ${convergence_json_path}" >&2
  exit 1
fi

kickoff_primary_focus="$(jq -r '.primaryFocus' "${kickoff_json_path}")"
plan_selected_track="$(jq -r '.selectedTrack' "${plan_json_path}")"
plan_selected_slice_id="$(jq -r '.plan[0].id' "${plan_json_path}")"
runtime_selected_track="$(jq -r '.selectedTrack' "${runtime_execution_json_path}")"
runtime_selected_slice_id="$(jq -r '.selectedSlice.id' "${runtime_execution_json_path}")"
runtime_status="$(jq -r '.executionStatus' "${runtime_execution_json_path}")"
convergence_selected_track="$(jq -r '.selectedTrack' "${convergence_json_path}")"
convergence_selected_slice_id="$(jq -r '.selectedSliceId' "${convergence_json_path}")"
convergence_overall="$(jq -r '.overall' "${convergence_json_path}")"
convergence_gate="$(jq -r '.convergenceClosureGate' "${convergence_json_path}")"

packet_kickoff_primary_focus="$(jq -r '.summary.kickoffPrimaryFocus' "${packet_json_path}")"
packet_plan_selected_track="$(jq -r '.summary.planSelectedTrack' "${packet_json_path}")"
packet_plan_selected_slice_id="$(jq -r '.summary.planSelectedSliceId' "${packet_json_path}")"
packet_runtime_status="$(jq -r '.summary.runtimeStatus' "${packet_json_path}")"
packet_convergence_overall="$(jq -r '.summary.convergenceOverall' "${packet_json_path}")"
packet_closure_gate="$(jq -r '.closureGate' "${packet_json_path}")"
packet_kickoff_artifact="$(jq -r '.artifacts.kickoff' "${packet_json_path}")"
packet_matrix_artifact="$(jq -r '.artifacts.priorityMatrix' "${packet_json_path}")"
packet_plan_artifact="$(jq -r '.artifacts.runtimePlan' "${packet_json_path}")"
packet_runtime_artifact="$(jq -r '.artifacts.runtimeExecution' "${packet_json_path}")"
packet_convergence_artifact="$(jq -r '.artifacts.convergence' "${packet_json_path}")"

if [ "${runtime_selected_track}" != "${plan_selected_track}" ]; then
  echo "runtime execution selectedTrack does not match runtime plan track: ${runtime_selected_track} != ${plan_selected_track}" >&2
  exit 1
fi
if [ "${runtime_selected_slice_id}" != "${plan_selected_slice_id}" ]; then
  echo "runtime execution selectedSlice.id does not match runtime plan slice id: ${runtime_selected_slice_id} != ${plan_selected_slice_id}" >&2
  exit 1
fi
if [ "${convergence_selected_track}" != "${plan_selected_track}" ]; then
  echo "convergence selectedTrack does not match runtime plan track: ${convergence_selected_track} != ${plan_selected_track}" >&2
  exit 1
fi
if [ "${convergence_selected_slice_id}" != "${plan_selected_slice_id}" ]; then
  echo "convergence selectedSliceId does not match runtime plan slice id: ${convergence_selected_slice_id} != ${plan_selected_slice_id}" >&2
  exit 1
fi

if [ "${packet_kickoff_primary_focus}" != "${kickoff_primary_focus}" ]; then
  echo "transition packet kickoffPrimaryFocus does not match kickoff primaryFocus: ${packet_kickoff_primary_focus} != ${kickoff_primary_focus}" >&2
  exit 1
fi
if [ "${packet_plan_selected_track}" != "${plan_selected_track}" ]; then
  echo "transition packet planSelectedTrack does not match runtime plan track: ${packet_plan_selected_track} != ${plan_selected_track}" >&2
  exit 1
fi
if [ "${packet_plan_selected_slice_id}" != "${plan_selected_slice_id}" ]; then
  echo "transition packet planSelectedSliceId does not match runtime plan slice id: ${packet_plan_selected_slice_id} != ${plan_selected_slice_id}" >&2
  exit 1
fi
if [ "${packet_runtime_status}" != "${runtime_status}" ]; then
  echo "transition packet runtimeStatus does not match runtime execution status: ${packet_runtime_status} != ${runtime_status}" >&2
  exit 1
fi
if [ "${packet_convergence_overall}" != "${convergence_overall}" ]; then
  echo "transition packet convergenceOverall does not match convergence overall: ${packet_convergence_overall} != ${convergence_overall}" >&2
  exit 1
fi

if [ "${packet_kickoff_artifact}" != "kickoff.json" ]; then
  echo "transition packet kickoff artifact is not canonical: ${packet_kickoff_artifact}" >&2
  exit 1
fi
if [ "${packet_matrix_artifact}" != "priority-matrix.json" ]; then
  echo "transition packet priorityMatrix artifact is not canonical: ${packet_matrix_artifact}" >&2
  exit 1
fi
if [ "${packet_plan_artifact}" != "runtime-plan.json" ]; then
  echo "transition packet runtimePlan artifact is not canonical: ${packet_plan_artifact}" >&2
  exit 1
fi
if [ "${packet_runtime_artifact}" != "runtime-execution.json" ]; then
  echo "transition packet runtimeExecution artifact is not canonical: ${packet_runtime_artifact}" >&2
  exit 1
fi
if [ "${packet_convergence_artifact}" != "convergence.json" ]; then
  echo "transition packet convergence artifact is not canonical: ${packet_convergence_artifact}" >&2
  exit 1
fi

if [ "${convergence_gate}" != "M36-E" ]; then
  echo "unexpected M36 convergence closure gate marker: ${convergence_gate}" >&2
  exit 1
fi
if [ "${packet_closure_gate}" != "M36-F" ]; then
  echo "unexpected M36 transition packet closure gate marker: ${packet_closure_gate}" >&2
  exit 1
fi

required_gates=(M36-A M36-B M36-C M36-D M36-E M36-F)
gates_json='[]'
all_m36_gates_pass="true"

for gate in "${required_gates[@]}"; do
  gate_status="$(printf '%s\n' "${closure_json}" | jq -r --arg gate "${gate}" '.gates[] | select(.gate == $gate) | .status' | head -n 1)"
  if [ -z "${gate_status}" ] || [ "${gate_status}" = "null" ]; then
    echo "missing required M36 gate in closure json: ${gate}" >&2
    exit 1
  fi
  if [ "${gate_status}" != "PASS" ]; then
    all_m36_gates_pass="false"
  fi
  gates_json="$(jq -c --arg gate "${gate}" --arg status "${gate_status}" '. + [{gate: $gate, status: $status}]' <<<"${gates_json}")"
done

runtime_ok="false"
if [ "${runtime_status}" = "PASS" ] || [ "${runtime_status}" = "DRY_RUN" ]; then
  runtime_ok="true"
fi

overall_status="PASS"
if [ "${all_m36_gates_pass}" != "true" ] || [ "${convergence_overall}" != "PASS" ] || [ "${runtime_ok}" != "true" ]; then
  overall_status="PENDING"
fi

next_action="M36 is closed; start M37 kickoff."
if [ "${overall_status}" != "PASS" ]; then
  next_action="Resolve pending M36 gates/packet convergence before starting M37."
fi

report_json="$(
  jq -n \
    --arg version "0.1" \
    --arg closureGate "M36-G" \
    --arg source "${closure_source}" \
    --arg kickoffJson "${kickoff_json_path}" \
    --arg matrixJson "${matrix_json_path}" \
    --arg planJson "${plan_json_path}" \
    --arg runtimeExecutionJson "${runtime_execution_json_path}" \
    --arg convergenceJson "${convergence_json_path}" \
    --arg packetJson "${packet_json_path}" \
    --arg overall "${overall_status}" \
    --arg kickoffPrimaryFocus "${kickoff_primary_focus}" \
    --arg planSelectedTrack "${plan_selected_track}" \
    --arg planSelectedSliceId "${plan_selected_slice_id}" \
    --arg runtimeStatus "${runtime_status}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg nextAction "${next_action}" \
    --argjson gates "${gates_json}" \
    '{
      version: $version,
      closureGate: $closureGate,
      source: $source,
      artifacts: {
        kickoffJson: $kickoffJson,
        matrixJson: $matrixJson,
        planJson: $planJson,
        runtimeExecutionJson: $runtimeExecutionJson,
        convergenceJson: $convergenceJson,
        packetJson: $packetJson
      },
      overall: $overall,
      m36Gates: $gates,
      packetSummary: {
        kickoffPrimaryFocus: $kickoffPrimaryFocus,
        planSelectedTrack: $planSelectedTrack,
        planSelectedSliceId: $planSelectedSliceId,
        runtimeStatus: $runtimeStatus,
        convergenceOverall: $convergenceOverall
      },
      nextAction: $nextAction
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${report_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"
{
  echo "# M36 Closure Report"
  echo
  printf -- "- closureGate: \`%s\`\n" "M36-G"
  printf -- "- source: \`%s\`\n" "${closure_source}"
  printf -- "- kickoffJson: \`%s\`\n" "${kickoff_json_path}"
  printf -- "- matrixJson: \`%s\`\n" "${matrix_json_path}"
  printf -- "- planJson: \`%s\`\n" "${plan_json_path}"
  printf -- "- runtimeExecutionJson: \`%s\`\n" "${runtime_execution_json_path}"
  printf -- "- convergenceJson: \`%s\`\n" "${convergence_json_path}"
  printf -- "- packetJson: \`%s\`\n" "${packet_json_path}"
  printf -- "- overall: \`%s\`\n" "${overall_status}"
  echo
  echo "| Gate | Status |"
  echo "| --- | --- |"
  jq -r '.m36Gates[] | "| \(.gate) | \(.status) |"' <<<"${report_json}"
  echo
  echo "## Packet Summary"
  printf -- "- kickoffPrimaryFocus: \`%s\`\n" "${kickoff_primary_focus}"
  printf -- "- planSelectedTrack: \`%s\`\n" "${plan_selected_track}"
  printf -- "- planSelectedSliceId: \`%s\`\n" "${plan_selected_slice_id}"
  printf -- "- runtimeStatus: \`%s\`\n" "${runtime_status}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  echo
  printf -- "- nextAction: %s\n" "${next_action}"
} > "${output_path}"

echo "m36 closure report generated: ${output_path}"
