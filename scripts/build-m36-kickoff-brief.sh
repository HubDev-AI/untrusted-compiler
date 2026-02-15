#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m35-closure-json <path>] [--m35-packet-json <path>] [--output <path>] [--format <text|json>]

Builds deterministic M36 kickoff brief from finalized M35 closure report + transition packet artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m35_closure_json_path="${root_dir}/build/m35-closure-report.json"
m35_packet_json_path="${root_dir}/build/m35-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m36-kickoff-brief.json"
output_format="text"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m35-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m35_closure_json_path="$2"
      shift 2
      ;;
    --m35-closure-json=*)
      m35_closure_json_path="${1#--m35-closure-json=}"
      shift
      ;;
    --m35-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m35_packet_json_path="$2"
      shift 2
      ;;
    --m35-packet-json=*)
      m35_packet_json_path="${1#--m35-packet-json=}"
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

if [ ! -f "${m35_closure_json_path}" ]; then
  echo "missing M35 closure report json: ${m35_closure_json_path}" >&2
  exit 1
fi
if [ ! -f "${m35_packet_json_path}" ]; then
  echo "missing M35 transition packet json: ${m35_packet_json_path}" >&2
  exit 1
fi

if ! jq -e '.overall and .closureGate and (.m35Gates | type == "array") and .packetSummary and .packetSummary.kickoffPrimaryFocus and .packetSummary.planSelectedTrack and .packetSummary.planSelectedSliceId and .packetSummary.runtimeStatus and .packetSummary.convergenceOverall' "${m35_closure_json_path}" >/dev/null; then
  echo "invalid M35 closure report json contract: ${m35_closure_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.kickoffPrimaryFocus and .summary.planSelectedTrack and .summary.planSelectedSliceId and .summary.runtimeStatus and .summary.convergenceOverall and .closureGate' "${m35_packet_json_path}" >/dev/null; then
  echo "invalid M35 transition packet json contract: ${m35_packet_json_path}" >&2
  exit 1
fi

m35_overall="$(jq -r '.overall' "${m35_closure_json_path}")"
m35_closure_gate="$(jq -r '.closureGate' "${m35_closure_json_path}")"
m35_packet_gate="$(jq -r '.closureGate' "${m35_packet_json_path}")"
pending_gates_json="$(jq -c '[.m35Gates[] | select(.status != "PASS") | .gate]' "${m35_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"

closure_kickoff_primary_focus="$(jq -r '.packetSummary.kickoffPrimaryFocus' "${m35_closure_json_path}")"
closure_plan_selected_track="$(jq -r '.packetSummary.planSelectedTrack' "${m35_closure_json_path}")"
closure_plan_selected_slice_id="$(jq -r '.packetSummary.planSelectedSliceId' "${m35_closure_json_path}")"
closure_runtime_status="$(jq -r '.packetSummary.runtimeStatus' "${m35_closure_json_path}")"
closure_convergence_overall="$(jq -r '.packetSummary.convergenceOverall' "${m35_closure_json_path}")"

packet_kickoff_primary_focus="$(jq -r '.summary.kickoffPrimaryFocus' "${m35_packet_json_path}")"
packet_plan_selected_track="$(jq -r '.summary.planSelectedTrack' "${m35_packet_json_path}")"
packet_plan_selected_slice_id="$(jq -r '.summary.planSelectedSliceId' "${m35_packet_json_path}")"
packet_runtime_status="$(jq -r '.summary.runtimeStatus' "${m35_packet_json_path}")"
packet_convergence_overall="$(jq -r '.summary.convergenceOverall' "${m35_packet_json_path}")"

if [ "${closure_kickoff_primary_focus}" != "${packet_kickoff_primary_focus}" ]; then
  echo "M35 closure report packetSummary.kickoffPrimaryFocus does not match transition packet summary.kickoffPrimaryFocus: ${closure_kickoff_primary_focus} != ${packet_kickoff_primary_focus}" >&2
  exit 1
fi
if [ "${closure_plan_selected_track}" != "${packet_plan_selected_track}" ]; then
  echo "M35 closure report packetSummary.planSelectedTrack does not match transition packet summary.planSelectedTrack: ${closure_plan_selected_track} != ${packet_plan_selected_track}" >&2
  exit 1
fi
if [ "${closure_plan_selected_slice_id}" != "${packet_plan_selected_slice_id}" ]; then
  echo "M35 closure report packetSummary.planSelectedSliceId does not match transition packet summary.planSelectedSliceId: ${closure_plan_selected_slice_id} != ${packet_plan_selected_slice_id}" >&2
  exit 1
fi
if [ "${closure_runtime_status}" != "${packet_runtime_status}" ]; then
  echo "M35 closure report packetSummary.runtimeStatus does not match transition packet summary.runtimeStatus: ${closure_runtime_status} != ${packet_runtime_status}" >&2
  exit 1
fi
if [ "${closure_convergence_overall}" != "${packet_convergence_overall}" ]; then
  echo "M35 closure report packetSummary.convergenceOverall does not match transition packet summary.convergenceOverall: ${closure_convergence_overall} != ${packet_convergence_overall}" >&2
  exit 1
fi

if [ "${m35_closure_gate}" != "M35-G" ]; then
  echo "unexpected M35 closure report gate marker: ${m35_closure_gate}" >&2
  exit 1
fi
if [ "${m35_packet_gate}" != "M35-F" ]; then
  echo "unexpected M35 transition packet gate marker: ${m35_packet_gate}" >&2
  exit 1
fi

primary_focus="stabilization"
if [ "${m35_overall}" = "PASS" ] && [ "${packet_convergence_overall}" = "PASS" ] && [ "${pending_gate_count}" -eq 0 ]; then
  primary_focus="${packet_plan_selected_track}"
fi

m36_kickoff_intent="Resolve remaining M35 closure/convergence issues before widening M36 scope."
recommendations_json='[]'

if [ "${primary_focus}" = "stabilization" ]; then
  if [ "${pending_gate_count}" -gt 0 ]; then
    recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
      "Keep M36 in stabilization mode until all M35 gates pass.",
      ("Resolve pending M35 gates first: " + ($pendingGates | join(", ")))
    ]')"
  else
    recommendations_json="$(jq -n '[
      "Keep M36 in stabilization mode until M35 closure and convergence return PASS.",
      "Regenerate M35 closure artifacts after resolving runtime and convergence drift."
    ]')"
  fi
else
  m36_kickoff_intent="Start M36 on the ${primary_focus} track and continue from M35 handoff slice ${packet_plan_selected_slice_id}."
  recommendations_json="$(jq -n \
    --arg primaryFocus "${primary_focus}" \
    --arg selectedSliceId "${packet_plan_selected_slice_id}" \
    --arg runtimeStatus "${packet_runtime_status}" \
    '[
      ("Open M36 with primary focus track: " + $primaryFocus + "."),
      ("Use M35 handoff slice as execution baseline: " + $selectedSliceId),
      ("Carry forward runtime status context from M35 handoff: " + $runtimeStatus)
    ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg kickoffGate "M36-A" \
    --arg m35ClosureJson "${m35_closure_json_path}" \
    --arg m35PacketJson "${m35_packet_json_path}" \
    --arg m35Overall "${m35_overall}" \
    --arg m35ClosureGate "${m35_closure_gate}" \
    --arg m35PacketGate "${m35_packet_gate}" \
    --arg kickoffPrimaryFocus "${packet_kickoff_primary_focus}" \
    --arg selectedTrack "${packet_plan_selected_track}" \
    --arg selectedSliceId "${packet_plan_selected_slice_id}" \
    --arg runtimeStatus "${packet_runtime_status}" \
    --arg convergenceOverall "${packet_convergence_overall}" \
    --arg primaryFocus "${primary_focus}" \
    --arg m36KickoffIntent "${m36_kickoff_intent}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson pendingGateCount "${pending_gate_count}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      kickoffGate: $kickoffGate,
      inputs: {
        m35ClosureJson: $m35ClosureJson,
        m35PacketJson: $m35PacketJson
      },
      m35Overall: $m35Overall,
      m35ClosureGate: $m35ClosureGate,
      m35PacketGate: $m35PacketGate,
      kickoffPrimaryFocus: $kickoffPrimaryFocus,
      selectedTrack: $selectedTrack,
      selectedSliceId: $selectedSliceId,
      runtimeStatus: $runtimeStatus,
      convergenceOverall: $convergenceOverall,
      pendingGates: $pendingGates,
      pendingGateCount: $pendingGateCount,
      primaryFocus: $primaryFocus,
      m36KickoffIntent: $m36KickoffIntent,
      recommendations: $recommendations
    }'
)"

mkdir -p "$(dirname "${output_path}")"
printf '%s\n' "${brief_json}" > "${output_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${brief_json}"
  exit 0
fi

echo "M36 Kickoff Brief"
echo "kickoffGate: M36-A"
echo "m35Overall: ${m35_overall}"
echo "m35ClosureGate: ${m35_closure_gate}"
echo "m35PacketGate: ${m35_packet_gate}"
echo "kickoffPrimaryFocus: ${packet_kickoff_primary_focus}"
echo "selectedTrack: ${packet_plan_selected_track}"
echo "selectedSliceId: ${packet_plan_selected_slice_id}"
echo "runtimeStatus: ${packet_runtime_status}"
echo "convergenceOverall: ${packet_convergence_overall}"
echo "pendingGateCount: ${pending_gate_count}"
echo "primaryFocus: ${primary_focus}"
echo "m36KickoffIntent: ${m36_kickoff_intent}"
echo "recommendations:"
jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
echo "output: ${output_path}"
