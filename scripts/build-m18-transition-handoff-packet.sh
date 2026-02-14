#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--matrix-json <path>] [--selector-json <path>] [--convergence-json <path>] [--output-dir <path>] [--format <text|json>]

Builds a deterministic transition handoff packet from M18 execution artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m18-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m18-priority-matrix.json"
selector_json_path="${root_dir}/build/m18-next-slice.json"
convergence_json_path=""
output_dir="${root_dir}/build/m18-transition-handoff"
output_format="text"

while [ "$#" -gt 0 ]; do
  case "$1" in
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
    --selector-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      selector_json_path="$2"
      shift 2
      ;;
    --selector-json=*)
      selector_json_path="${1#--selector-json=}"
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

require_file() {
  local path="$1"
  local label="$2"
  if [ ! -f "${path}" ]; then
    echo "missing ${label}: ${path}" >&2
    exit 1
  fi
}

require_contract() {
  local path="$1"
  local jq_expr="$2"
  local label="$3"
  if ! jq -e "${jq_expr}" "${path}" >/dev/null; then
    echo "invalid ${label} contract: ${path}" >&2
    exit 1
  fi
}

require_file "${kickoff_json_path}" "kickoff json"
require_file "${matrix_json_path}" "priority matrix json"
require_file "${selector_json_path}" "selector json"

require_contract "${kickoff_json_path}" '.overall and .failedStep and .frictionCount' "kickoff json"
require_contract "${matrix_json_path}" '.overall and (.tracks | type == "array") and (.tracks | length > 0)' "priority matrix json"
require_contract "${selector_json_path}" '.selectedTrack and .recommendation and .recommendation.id' "selector json"

if [ -z "${convergence_json_path}" ]; then
  convergence_json_path="${output_dir}/convergence.generated.json"
  "${root_dir}/scripts/build-m18-track-convergence-summary.sh" --format json > "${convergence_json_path}"
fi

require_file "${convergence_json_path}" "convergence json"
require_contract "${convergence_json_path}" '.overall and (.tracks | type == "array") and (.tracks | length == 3)' "convergence json"

mkdir -p "${output_dir}"

packet_kickoff="${output_dir}/kickoff.json"
packet_matrix="${output_dir}/priority-matrix.json"
packet_selector="${output_dir}/selector.json"
packet_convergence="${output_dir}/convergence.json"
packet_manifest="${output_dir}/handoff-packet.json"

cp "${kickoff_json_path}" "${packet_kickoff}"
cp "${matrix_json_path}" "${packet_matrix}"
cp "${selector_json_path}" "${packet_selector}"
cp "${convergence_json_path}" "${packet_convergence}"

generated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
kickoff_overall="$(jq -r '.overall' "${packet_kickoff}")"
selector_track="$(jq -r '.selectedTrack' "${packet_selector}")"
selector_recommendation_id="$(jq -r '.recommendation.id' "${packet_selector}")"
convergence_overall="$(jq -r '.overall' "${packet_convergence}")"

jq -n \
  --arg version "0.1" \
  --arg generatedAt "${generated_at}" \
  --arg kickoffOverall "${kickoff_overall}" \
  --arg selectorTrack "${selector_track}" \
  --arg selectorRecommendationId "${selector_recommendation_id}" \
  --arg convergenceOverall "${convergence_overall}" \
  --arg kickoffFile "$(basename "${packet_kickoff}")" \
  --arg matrixFile "$(basename "${packet_matrix}")" \
  --arg selectorFile "$(basename "${packet_selector}")" \
  --arg convergenceFile "$(basename "${packet_convergence}")" \
  '{
    version: $version,
    generatedAt: $generatedAt,
    summary: {
      kickoffOverall: $kickoffOverall,
      selectorTrack: $selectorTrack,
      selectorRecommendationId: $selectorRecommendationId,
      convergenceOverall: $convergenceOverall
    },
    artifacts: {
      kickoff: $kickoffFile,
      priorityMatrix: $matrixFile,
      selector: $selectorFile,
      convergence: $convergenceFile
    }
  }' > "${packet_manifest}"

if [ "${output_format}" = "json" ]; then
  cat "${packet_manifest}"
  exit 0
fi

echo "M18 transition handoff packet generated"
echo "outputDir: ${output_dir}"
echo "manifest: ${packet_manifest}"
echo "kickoffOverall: ${kickoff_overall}"
echo "selectorTrack: ${selector_track}"
echo "selectorRecommendationId: ${selector_recommendation_id}"
echo "convergenceOverall: ${convergence_overall}"
