#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--m24-closure-json <path>] [--m24-packet-json <path>] [--output <path>] [--format <markdown|json>]

Generates an M25 kickoff brief from finalized M24 closure + transition packet artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
m24_closure_json_path="${root_dir}/build/m24-closure-report.json"
m24_packet_json_path="${root_dir}/build/m24-transition-handoff/handoff-packet.json"
output_path="${root_dir}/build/m25-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --m24-closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m24_closure_json_path="$2"
      shift 2
      ;;
    --m24-closure-json=*)
      m24_closure_json_path="${1#--m24-closure-json=}"
      shift
      ;;
    --m24-packet-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      m24_packet_json_path="$2"
      shift 2
      ;;
    --m24-packet-json=*)
      m24_packet_json_path="${1#--m24-packet-json=}"
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

if [ ! -f "${m24_packet_json_path}" ]; then
  echo "missing M24 transition packet json: ${m24_packet_json_path}" >&2
  exit 1
fi
if ! jq -e '.summary and .summary.selectorTrack and .summary.selectorRecommendationId and .summary.convergenceOverall' "${m24_packet_json_path}" >/dev/null; then
  echo "invalid M24 transition packet json contract: ${m24_packet_json_path}" >&2
  exit 1
fi

if [ ! -f "${m24_closure_json_path}" ]; then
  mkdir -p "$(dirname "${m24_closure_json_path}")"
  "${root_dir}/scripts/build-m24-closure-report.sh" --packet-json "${m24_packet_json_path}" --format json > "${m24_closure_json_path}"
fi

if ! jq -e '.overall and (.m24Gates | type == "array") and .packetSummary and .packetSummary.convergenceOverall' "${m24_closure_json_path}" >/dev/null; then
  echo "invalid M24 closure report json contract: ${m24_closure_json_path}" >&2
  exit 1
fi

m24_overall="$(jq -r '.overall' "${m24_closure_json_path}")"
pending_gates_json="$(jq -c '[.m24Gates[] | select(.status != "PASS") | .gate]' "${m24_closure_json_path}")"
pending_gate_count="$(jq -r 'length' <<<"${pending_gates_json}")"
selector_track="$(jq -r '.summary.selectorTrack' "${m24_packet_json_path}")"
selector_recommendation_id="$(jq -r '.summary.selectorRecommendationId' "${m24_packet_json_path}")"
convergence_overall="$(jq -r '.summary.convergenceOverall' "${m24_packet_json_path}")"

primary_focus="stabilization"
if [ "${m24_overall}" = "PASS" ] && [ "${convergence_overall}" = "PASS" ]; then
  primary_focus="${selector_track}"
fi

recommendations_json='[]'
if [ "${primary_focus}" = "stabilization" ]; then
  recommendations_json="$(jq -n --argjson pendingGates "${pending_gates_json}" '[
    "Resolve pending M24 gates before opening broader M25 scope.",
    ("Address pending gates first: " + ($pendingGates | join(", ")))
  ]')"
else
  recommendations_json="$(jq -n --arg primaryFocus "${primary_focus}" --arg selectorRecommendation "${selector_recommendation_id}" '[
    ("Start M25 with primary focus track: " + $primaryFocus + "."),
    ("Use final M24 selector recommendation as baseline: " + $selectorRecommendation)
  ]')"
fi

brief_json="$(
  jq -n \
    --arg version "0.1" \
    --arg m24ClosureJson "${m24_closure_json_path}" \
    --arg m24PacketJson "${m24_packet_json_path}" \
    --arg m24Overall "${m24_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg selectorTrack "${selector_track}" \
    --arg selectorRecommendationId "${selector_recommendation_id}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGates "${pending_gates_json}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      inputs: {
        m24ClosureJson: $m24ClosureJson,
        m24PacketJson: $m24PacketJson
      },
      m24Overall: $m24Overall,
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
  echo "# M25 Kickoff Brief"
  echo
  printf -- "- M24 closure json: \`%s\`\n" "${m24_closure_json_path}"
  printf -- "- M24 packet json: \`%s\`\n" "${m24_packet_json_path}"
  printf -- "- m24Overall: \`%s\`\n" "${m24_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- selectorTrack: \`%s\`\n" "${selector_track}"
  printf -- "- selectorRecommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${brief_json}"
} > "${output_path}"

echo "m25 kickoff brief generated: ${output_path}"
