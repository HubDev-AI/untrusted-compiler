#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--matrix-json <path>] [--selector-json <path>] [--runtime-execution-json <path>] [--convergence-json <path>] [--output-dir <path>] [--format <text|json>]

Builds a deterministic M19 transition handoff packet from executed-slice artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m19-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m19-priority-matrix.json"
selector_json_path="${root_dir}/build/m19-next-slice.json"
runtime_execution_json_path="${root_dir}/build/m19-runtime-hardening.json"
convergence_json_path=""
output_dir="${root_dir}/build/m19-transition-handoff"
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
require_file "${runtime_execution_json_path}" "runtime execution json"

require_contract "${kickoff_json_path}" '.m18Overall and .primaryFocus and (.pendingGates | type == "array")' "kickoff json"
require_contract "${matrix_json_path}" '.tracks and (.tracks | type == "array") and (.tracks | length > 0)' "priority matrix json"
require_contract "${selector_json_path}" '.selectedTrack and .recommendation and .recommendation.id' "selector json"
require_contract "${runtime_execution_json_path}" '.selectedTrack and .recommendationId and .status' "runtime execution json"

selector_track="$(jq -r '.selectedTrack' "${selector_json_path}")"
selector_recommendation_id="$(jq -r '.recommendation.id' "${selector_json_path}")"
runtime_track="$(jq -r '.selectedTrack' "${runtime_execution_json_path}")"
runtime_recommendation_id="$(jq -r '.recommendationId' "${runtime_execution_json_path}")"
if [ "${runtime_track}" != "${selector_track}" ]; then
  echo "runtime execution selectedTrack does not match selector track: ${runtime_track} != ${selector_track}" >&2
  exit 1
fi
if [ "${runtime_recommendation_id}" != "${selector_recommendation_id}" ]; then
  echo "runtime execution recommendationId does not match selector recommendation id: ${runtime_recommendation_id} != ${selector_recommendation_id}" >&2
  exit 1
fi

mkdir -p "${output_dir}"

if [ -z "${convergence_json_path}" ]; then
  convergence_json_path="${output_dir}/convergence.generated.json"
  "${root_dir}/scripts/build-m19-executed-slice-convergence-summary.sh" \
    --selector-json "${selector_json_path}" \
    --runtime-execution-json "${runtime_execution_json_path}" \
    --format json > "${convergence_json_path}"
fi

require_file "${convergence_json_path}" "convergence json"
require_contract "${convergence_json_path}" '.overall and .selectedTrack and .recommendationId' "convergence json"

packet_kickoff="${output_dir}/kickoff.json"
packet_matrix="${output_dir}/priority-matrix.json"
packet_selector="${output_dir}/selector.json"
packet_runtime_execution="${output_dir}/runtime-execution.json"
packet_convergence="${output_dir}/convergence.json"
packet_manifest="${output_dir}/handoff-packet.json"

cp "${kickoff_json_path}" "${packet_kickoff}"
cp "${matrix_json_path}" "${packet_matrix}"
cp "${selector_json_path}" "${packet_selector}"
cp "${runtime_execution_json_path}" "${packet_runtime_execution}"
cp "${convergence_json_path}" "${packet_convergence}"

generated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
kickoff_primary_focus="$(jq -r '.primaryFocus' "${packet_kickoff}")"
runtime_status="$(jq -r '.status' "${packet_runtime_execution}")"
convergence_overall="$(jq -r '.overall' "${packet_convergence}")"

jq -n \
  --arg version "0.1" \
  --arg generatedAt "${generated_at}" \
  --arg kickoffPrimaryFocus "${kickoff_primary_focus}" \
  --arg selectorTrack "${selector_track}" \
  --arg selectorRecommendationId "${selector_recommendation_id}" \
  --arg runtimeStatus "${runtime_status}" \
  --arg convergenceOverall "${convergence_overall}" \
  --arg kickoffFile "$(basename "${packet_kickoff}")" \
  --arg matrixFile "$(basename "${packet_matrix}")" \
  --arg selectorFile "$(basename "${packet_selector}")" \
  --arg runtimeExecutionFile "$(basename "${packet_runtime_execution}")" \
  --arg convergenceFile "$(basename "${packet_convergence}")" \
  '{
    version: $version,
    generatedAt: $generatedAt,
    summary: {
      kickoffPrimaryFocus: $kickoffPrimaryFocus,
      selectorTrack: $selectorTrack,
      selectorRecommendationId: $selectorRecommendationId,
      runtimeStatus: $runtimeStatus,
      convergenceOverall: $convergenceOverall
    },
    artifacts: {
      kickoff: $kickoffFile,
      priorityMatrix: $matrixFile,
      selector: $selectorFile,
      runtimeExecution: $runtimeExecutionFile,
      convergence: $convergenceFile
    }
  }' > "${packet_manifest}"

if [ "${output_format}" = "json" ]; then
  cat "${packet_manifest}"
  exit 0
fi

echo "M19 transition handoff packet generated"
echo "outputDir: ${output_dir}"
echo "manifest: ${packet_manifest}"
echo "kickoffPrimaryFocus: ${kickoff_primary_focus}"
echo "selectorTrack: ${selector_track}"
echo "selectorRecommendationId: ${selector_recommendation_id}"
echo "runtimeStatus: ${runtime_status}"
echo "convergenceOverall: ${convergence_overall}"
