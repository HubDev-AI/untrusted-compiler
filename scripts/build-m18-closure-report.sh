#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--closure-json <path>] [--packet-json <path>] [--output <path>] [--format <markdown|json>]

Builds a final M18 closure report from closure gates and transition packet summary.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
closure_json_path=""
packet_json_path="${root_dir}/build/m18-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m18-closure-report.md"
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

if [ ! -f "${packet_json_path}" ]; then
  echo "missing transition packet json: ${packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.kickoffOverall and .summary.selectorTrack and .summary.convergenceOverall' "${packet_json_path}" >/dev/null; then
  echo "invalid transition packet json contract: ${packet_json_path}" >&2
  exit 1
fi

required_gates=(M18-A M18-B M18-C M18-D M18-E M18-F M18-G M18-H)
gates_json='[]'
all_m18_gates_pass="true"

for gate in "${required_gates[@]}"; do
  gate_status="$(printf '%s\n' "${closure_json}" | jq -r --arg gate "${gate}" '.gates[] | select(.gate == $gate) | .status' | head -n 1)"
  if [ -z "${gate_status}" ] || [ "${gate_status}" = "null" ]; then
    echo "missing required M18 gate in closure json: ${gate}" >&2
    exit 1
  fi
  if [ "${gate_status}" != "PASS" ]; then
    all_m18_gates_pass="false"
  fi
  gates_json="$(jq -c --arg gate "${gate}" --arg status "${gate_status}" '. + [{gate: $gate, status: $status}]' <<<"${gates_json}")"
done

kickoff_overall="$(jq -r '.summary.kickoffOverall' "${packet_json_path}")"
selector_track="$(jq -r '.summary.selectorTrack' "${packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.selectorRecommendationId' "${packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${packet_json_path}")"

overall_status="PASS"
if [ "${all_m18_gates_pass}" != "true" ] || [ "${convergence_overall}" != "PASS" ]; then
  overall_status="PENDING"
fi

next_action="M18 is closed; start M19 kickoff."
if [ "${overall_status}" != "PASS" ]; then
  next_action="Resolve pending M18 gates or convergence before starting M19."
fi

report_json="$(
  jq -n \
    --arg version "0.1" \
    --arg source "${closure_source}" \
    --arg packetJson "${packet_json_path}" \
    --arg overall "${overall_status}" \
    --arg kickoffOverall "${kickoff_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg nextAction "${next_action}" \
    --argjson gates "${gates_json}" \
    '{
      version: $version,
      source: $source,
      packetJson: $packetJson,
      overall: $overall,
      m18Gates: $gates,
      packetSummary: {
        kickoffOverall: $kickoffOverall,
        selectorTrack: $selectorTrack,
        selectorRecommendationId: $selectorRecommendationId,
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
  echo "# M18 Closure Report"
  echo
  printf -- "- source: \`%s\`\n" "${closure_source}"
  printf -- "- packetJson: \`%s\`\n" "${packet_json_path}"
  printf -- "- overall: \`%s\`\n" "${overall_status}"
  echo
  echo "| Gate | Status |"
  echo "| --- | --- |"
  jq -r '.m18Gates[] | "| \(.gate) | \(.status) |"' <<<"${report_json}"
  echo
  echo "## Packet Summary"
  printf -- "- kickoffOverall: \`%s\`\n" "${kickoff_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  echo
  printf -- "- nextAction: %s\n" "${next_action}"
} > "${output_path}"

echo "m18 closure report generated: ${output_path}"
