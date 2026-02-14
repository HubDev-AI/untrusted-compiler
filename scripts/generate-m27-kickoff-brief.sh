#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m26-closure-json <path>] [--m26-packet-json <path>] [--output <path>] [--format <markdown|json>]

Generates an M27 kickoff brief from finalized M26 closure + transition packet artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m26_closure_json_path="${root_dir}/build/m26-closure-report.json"
m26_packet_json_path="${root_dir}/build/m26-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m27-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m26-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m26_closure_json_path="$2"
      shift 2
      ;;
    --m26-closure-json=*)
      m26_closure_json_path="${1#--m26-closure-json=}"
      shift
      ;;
    --m26-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m26_packet_json_path="$2"
      shift 2
      ;;
    --m26-packet-json=*)
      m26_packet_json_path="${1#--m26-packet-json=}"
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

if [ ! -f "${m26_packet_json_path}" ]; then
  echo "missing M26 transition packet json: ${m26_packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.selectorTrack and .summary.selectorRecommendationId and .summary.convergenceOverall' "${m26_packet_json_path}" >/dev/null; then
  echo "invalid M26 transition packet json contract: ${m26_packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${m26_closure_json_path}" ]; then
  mkdir -p "$(dirname "${m26_closure_json_path}")"
  "${root_dir}/scripts/build-m26-closure-report.sh" --packet-json "${m26_packet_json_path}" --format json > "${m26_closure_json_path}"
fi

if ! jq -e '.overall and (.m26Gates | type == "array") and .packetSummary and .packetSummary.convergenceOverall' "${m26_closure_json_path}" >/dev/null; then
  echo "invalid M26 closure report json contract: ${m26_closure_json_path}" >&2
  exit 1
fi

m26_overall="$(jq -r '.overall' "${m26_closure_json_path}")"
pending_gates_json="$(jq -c '[.m26Gates[] | select(.status != "PASS") | .gate]' "${m26_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"
selector_track="$(jq -r '.summary.selectorTrack' "${m26_packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.selectorRecommendationId' "${m26_packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${m26_packet_json_path}")"

primary_focus="stabilization"
if [ "${m26_overall}" = "PASS" ] && [ "${convergence_overall}" = "PASS" ]; then
  primary_focus="${selector_track}"
fi

recommendations_json='[]'
if [ "${primary_focus}" = "stabilization" ]; then
  recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
    "Resolve pending M26 gates before opening broader M27 scope.",
    ("Address pending gates first: " + ($pendingGates | join(", ")))
  ]')"
else
  recommendations_json="$(jq -n --arg primaryFocus "${primary_focus}" --arg selectorRecommendation "${selector_recommendation_id}" '[
    ("Start M27 with primary focus track: " + $primaryFocus + "."),
    ("Use final M26 selector recommendation as baseline: " + $selectorRecommendation)
  ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg m26ClosureJson "${m26_closure_json_path}" \
    --arg m26PacketJson "${m26_packet_json_path}" \
    --arg m26Overall "${m26_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      inputs: {
        m26ClosureJson: $m26ClosureJson,
        m26PacketJson: $m26PacketJson
      },
      m26Overall: $m26Overall,
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
  echo "# M27 Kickoff Brief"
  echo
  printf -- "- M26 closure json: \`%s\`\n" "${m26_closure_json_path}"
  printf -- "- M26 packet json: \`%s\`\n" "${m26_packet_json_path}"
  printf -- "- m26Overall: \`%s\`\n" "${m26_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
} > "${output_path}"

echo "m27 kickoff brief generated: ${output_path}"
