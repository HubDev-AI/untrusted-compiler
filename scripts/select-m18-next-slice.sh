#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--matrix-json <path>] [--output <path>] [--format <text|json>]

Selects the next M18 closure-gated slice recommendation.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m18-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m18-priority-matrix.json"
output_path="${root_dir}/build/m18-next-slice.json"
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

if [ ! -f "${kickoff_json_path}" ]; then
  echo "missing kickoff summary json: ${kickoff_json_path}" >&2
  exit 1
fi
if [ ! -f "${matrix_json_path}" ]; then
  echo "missing priority matrix json: ${matrix_json_path}" >&2
  exit 1
fi

if ! jq -e '.overall and .failedStep and .frictionCount' "${kickoff_json_path}" >/dev/null; then
  echo "invalid kickoff summary contract: ${kickoff_json_path}" >&2
  exit 1
fi
if ! jq -e '.overall and (.tracks | type == "array") and (.tracks | length > 0)' "${matrix_json_path}" >/dev/null; then
  echo "invalid priority matrix contract: ${matrix_json_path}" >&2
  exit 1
fi

overall="$(jq -r '.overall' "${kickoff_json_path}")"
failed_step="$(jq -r '.failedStep' "${kickoff_json_path}")"
friction_count="$(jq -r '.frictionCount' "${kickoff_json_path}")"
top_track="$(jq -r '.tracks[0].track' "${matrix_json_path}")"

selected_track="${top_track}"
slice_id=""
slice_title=""

if [ "${overall}" != "PASS" ] || [ "${friction_count}" -gt 0 ]; then
  selected_track="runtime"
  slice_id="M18-S6-runtime-remediation-first"
  slice_title="Resolve runtime/flow friction before expanding editor or release scope"
else
  case "${top_track}" in
    editor)
      slice_id="M18-S4-editor-contract-expansion"
      slice_title="Expand editor and tooling contract coverage on stable runtime baseline"
      ;;
    release)
      slice_id="M18-S5-release-publish-integrity-contract-expansion"
      slice_title="Advance release/publish integrity contract checks from stable rehearsal baseline"
      ;;
    runtime)
      slice_id="M18-S6-runtime-confidence-hardening"
      slice_title="Increase runtime confidence guardrails despite passing baseline"
      ;;
    *)
      echo "unsupported top priority track: ${top_track}" >&2
      exit 1
      ;;
  esac
fi

selector_json="$(
  jq -n \
    --arg version "0.1" \
    --arg kickoffJson "${kickoff_json_path}" \
    --arg matrixJson "${matrix_json_path}" \
    --arg overall "${overall}" \
    --arg failedStep "${failed_step}" \
    --argjson frictionCount "${friction_count}" \
    --arg selectedTrack "${selected_track}" \
    --arg sliceId "${slice_id}" \
    --arg sliceTitle "${slice_title}" \
    --arg closureGate "M18-C" \
    '{
      version: $version,
      inputs: {
        kickoffJson: $kickoffJson,
        matrixJson: $matrixJson
      },
      overall: $overall,
      failedStep: $failedStep,
      frictionCount: $frictionCount,
      selectedTrack: $selectedTrack,
      recommendation: {
        id: $sliceId,
        title: $sliceTitle,
        closureGate: $closureGate
      }
    }'
)"

mkdir -p "$(dirname "${output_path}")"
printf '%s\n' "${selector_json}" > "${output_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${selector_json}"
  exit 0
fi

echo "M18 Next Slice Recommendation"
echo "overall: ${overall}"
echo "selectedTrack: ${selected_track}"
echo "sliceId: ${slice_id}"
echo "closureGate: M18-C"
echo "output: ${output_path}"
