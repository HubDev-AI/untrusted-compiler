#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m34-closure-json <path>] [--m34-packet-json <path>] [--output <path>] [--format <markdown|json>]

Generates an M35 kickoff brief from finalized M34 closure + transition packet artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m34_closure_json_path="${root_dir}/build/m34-closure-report.json"
m34_packet_json_path="${root_dir}/build/m34-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m35-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m34-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m34_closure_json_path="$2"
      shift 2
      ;;
    --m34-closure-json=*)
      m34_closure_json_path="${1#--m34-closure-json=}"
      shift
      ;;
    --m34-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m34_packet_json_path="$2"
      shift 2
      ;;
    --m34-packet-json=*)
      m34_packet_json_path="${1#--m34-packet-json=}"
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

if [ ! -f "${m34_packet_json_path}" ]; then
  echo "missing M34 transition packet json: ${m34_packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.planSelectedTrack and .summary.planSelectedSliceId and .summary.convergenceOverall' "${m34_packet_json_path}" >/dev/null; then
  echo "invalid M34 transition packet json contract: ${m34_packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${m34_closure_json_path}" ]; then
  mkdir -p "$(dirname "${m34_closure_json_path}")"
  "${root_dir}/scripts/build-m34-closure-report.sh" --packet-json "${m34_packet_json_path}" --format json > "${m34_closure_json_path}"
fi

if ! jq -e '.overall and (.m34Gates | type == "array") and .packetSummary and .packetSummary.convergenceOverall' "${m34_closure_json_path}" >/dev/null; then
  echo "invalid M34 closure report json contract: ${m34_closure_json_path}" >&2
  exit 1
fi

m34_overall="$(jq -r '.overall' "${m34_closure_json_path}")"
pending_gates_json="$(jq -c '[.m34Gates[] | select(.status != "PASS") | .gate]' "${m34_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"
selector_track="$(jq -r '.summary.planSelectedTrack' "${m34_packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.planSelectedSliceId' "${m34_packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${m34_packet_json_path}")"

primary_focus="stabilization"
if [ "${m34_overall}" = "PASS" ] && [ "${convergence_overall}" = "PASS" ]; then
  primary_focus="${selector_track}"
fi

recommendations_json='[]'
if [ "${primary_focus}" = "stabilization" ]; then
  recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
    "Resolve pending M34 gates before opening broader M35 scope.",
    ("Address pending M34 gates first: " + ($pendingGates | join(", ")))
  ]')"
else
  recommendations_json="$(jq -n --arg primaryFocus "${primary_focus}" --arg selectorRecommendation "${selector_recommendation_id}" '[
    ("Start M35 with primary focus track: " + $primaryFocus + "."),
    ("Use final M34 selector recommendation as baseline: " + $selectorRecommendation)
  ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg m34ClosureJson "${m34_closure_json_path}" \
    --arg m34PacketJson "${m34_packet_json_path}" \
    --arg m34Overall "${m34_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      inputs: {
        m34ClosureJson: $m34ClosureJson,
        m34PacketJson: $m34PacketJson
      },
      m34Overall: $m34Overall,
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
  echo "# M35 Kickoff Brief"
  echo
  printf -- "- M34 closure json: \`%s\`\n" "${m34_closure_json_path}"
  printf -- "- M34 packet json: \`%s\`\n" "${m34_packet_json_path}"
  printf -- "- m34Overall: \`%s\`\n" "${m34_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
} > "${output_path}"

echo "m35 kickoff brief generated: ${output_path}"
