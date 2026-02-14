#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m18-closure-json <path>] [--m18-packet-json <path>] [--output <path>] [--format <markdown|json>]

Generates an M19 kickoff brief from finalized M18 closure + handoff artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m18_closure_json_path="${root_dir}/build/m18-closure-report.json"
m18_packet_json_path="${root_dir}/build/m18-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m19-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m18-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m18_closure_json_path="$2"
      shift 2
      ;;
    --m18-closure-json=*)
      m18_closure_json_path="${1#--m18-closure-json=}"
      shift
      ;;
    --m18-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m18_packet_json_path="$2"
      shift 2
      ;;
    --m18-packet-json=*)
      m18_packet_json_path="${1#--m18-packet-json=}"
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

if [ ! -f "${m18_packet_json_path}" ]; then
  echo "missing M18 transition packet json: ${m18_packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.selectorTrack and .summary.selectorRecommendationId and .summary.convergenceOverall' "${m18_packet_json_path}" >/dev/null; then
  echo "invalid M18 transition packet json contract: ${m18_packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${m18_closure_json_path}" ]; then
  mkdir -p "$(dirname "${m18_closure_json_path}")"
  "${root_dir}/scripts/build-m18-closure-report.sh" --packet-json "${m18_packet_json_path}" --format json > "${m18_closure_json_path}"
fi

if ! jq -e '.overall and (.m18Gates | type == "array") and .packetSummary and .packetSummary.convergenceOverall' "${m18_closure_json_path}" >/dev/null; then
  echo "invalid M18 closure report json contract: ${m18_closure_json_path}" >&2
  exit 1
fi

m18_overall="$(jq -r '.overall' "${m18_closure_json_path}")"
pending_gates_json="$(jq -c '[.m18Gates[] | select(.status != "PASS") | .gate]' "${m18_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"
selector_track="$(jq -r '.summary.selectorTrack' "${m18_packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.selectorRecommendationId' "${m18_packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${m18_packet_json_path}")"

primary_focus="stabilization"
if [ "${m18_overall}" = "PASS" ] && [ "${convergence_overall}" = "PASS" ]; then
  primary_focus="${selector_track}"
fi

recommendations_json='[]'
if [ "${primary_focus}" = "stabilization" ]; then
  recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
    "Resolve pending M18 gates before opening broader M19 scope.",
    ("Address pending gates first: " + ($pendingGates | join(", ")))
  ]')"
else
  recommendations_json="$(jq -n --arg primaryFocus "${primary_focus}" --arg selectorRecommendation "${selector_recommendation_id}" '[
    ("Start M19 with primary focus track: " + $primaryFocus + "."),
    ("Use final M18 selector recommendation as baseline: " + $selectorRecommendation)
  ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg m18ClosureJson "${m18_closure_json_path}" \
    --arg m18PacketJson "${m18_packet_json_path}" \
    --arg m18Overall "${m18_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      inputs: {
        m18ClosureJson: $m18ClosureJson,
        m18PacketJson: $m18PacketJson
      },
      m18Overall: $m18Overall,
      convergenceOverall: $convergenceOverall,
      selectorTrack: $selectorTrack,
      selectorRecommendationId: $selectorRecommendationId,
      primaryFocus: $primaryFocus,
      pendingGates: $pendingGates,
      recommendations: $recommendations
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${brief_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"
{
  echo "# M19 Kickoff Brief"
  echo
  printf -- "- M18 closure json: \`%s\`\n" "${m18_closure_json_path}"
  printf -- "- M18 packet json: \`%s\`\n" "${m18_packet_json_path}"
  printf -- "- m18Overall: \`%s\`\n" "${m18_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
} > "${output_path}"

echo "m19 kickoff brief generated: ${output_path}"
