#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m25-closure-json <path>] [--m25-packet-json <path>] [--output <path>] [--format <markdown|json>]

Generates an M26 kickoff brief from finalized M25 closure + transition packet artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m25_closure_json_path="${root_dir}/build/m25-closure-report.json"
m25_packet_json_path="${root_dir}/build/m25-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m26-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m25-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m25_closure_json_path="$2"
      shift 2
      ;;
    --m25-closure-json=*)
      m25_closure_json_path="${1#--m25-closure-json=}"
      shift
      ;;
    --m25-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m25_packet_json_path="$2"
      shift 2
      ;;
    --m25-packet-json=*)
      m25_packet_json_path="${1#--m25-packet-json=}"
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

if [ ! -f "${m25_packet_json_path}" ]; then
  echo "missing M25 transition packet json: ${m25_packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.selectorTrack and .summary.selectorRecommendationId and .summary.convergenceOverall' "${m25_packet_json_path}" >/dev/null; then
  echo "invalid M25 transition packet json contract: ${m25_packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${m25_closure_json_path}" ]; then
  mkdir -p "$(dirname "${m25_closure_json_path}")"
  "${root_dir}/scripts/build-m25-closure-report.sh" --packet-json "${m25_packet_json_path}" --format json > "${m25_closure_json_path}"
fi

if ! jq -e '.overall and (.m25Gates | type == "array") and .packetSummary and .packetSummary.convergenceOverall' "${m25_closure_json_path}" >/dev/null; then
  echo "invalid M25 closure report json contract: ${m25_closure_json_path}" >&2
  exit 1
fi

m25_overall="$(jq -r '.overall' "${m25_closure_json_path}")"
pending_gates_json="$(jq -c '[.m25Gates[] | select(.status != "PASS") | .gate]' "${m25_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"
selector_track="$(jq -r '.summary.selectorTrack' "${m25_packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.selectorRecommendationId' "${m25_packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${m25_packet_json_path}")"

primary_focus="stabilization"
if [ "${m25_overall}" = "PASS" ] && [ "${convergence_overall}" = "PASS" ]; then
  primary_focus="${selector_track}"
fi

recommendations_json='[]'
if [ "${primary_focus}" = "stabilization" ]; then
  recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
    "Resolve pending M25 gates before opening broader M26 scope.",
    ("Address pending gates first: " + ($pendingGates | join(", ")))
  ]')"
else
  recommendations_json="$(jq -n --arg primaryFocus "${primary_focus}" --arg selectorRecommendation "${selector_recommendation_id}" '[
    ("Start M26 with primary focus track: " + $primaryFocus + "."),
    ("Use final M25 selector recommendation as baseline: " + $selectorRecommendation)
  ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg m25ClosureJson "${m25_closure_json_path}" \
    --arg m25PacketJson "${m25_packet_json_path}" \
    --arg m25Overall "${m25_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      inputs: {
        m25ClosureJson: $m25ClosureJson,
        m25PacketJson: $m25PacketJson
      },
      m25Overall: $m25Overall,
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
  echo "# M26 Kickoff Brief"
  echo
  printf -- "- M25 closure json: \`%s\`\n" "${m25_closure_json_path}"
  printf -- "- M25 packet json: \`%s\`\n" "${m25_packet_json_path}"
  printf -- "- m25Overall: \`%s\`\n" "${m25_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
} > "${output_path}"

echo "m26 kickoff brief generated: ${output_path}"
