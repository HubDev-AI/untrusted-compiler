#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--matrix-json <path>] [--output <path>] [--format <text|json>]

Selects the first executable M19 slice from kickoff + priority matrix artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m19-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m19-priority-matrix.json"
output_path="${root_dir}/build/m19-next-slice.json"
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
  echo "missing M19 kickoff json: ${kickoff_json_path}" >&2
  exit 1
fi
if [ ! -f "${matrix_json_path}" ]; then
  echo "missing M19 priority matrix json: ${matrix_json_path}" >&2
  exit 1
fi

if ! jq -e '.m18Overall and .primaryFocus and (.pendingGates | type == "array")' "${kickoff_json_path}" >/dev/null; then
  echo "invalid M19 kickoff json contract: ${kickoff_json_path}" >&2
  exit 1
fi
if ! jq -e '.tracks and (.tracks | type == "array") and (.tracks | length > 0)' "${matrix_json_path}" >/dev/null; then
  echo "invalid M19 priority matrix contract: ${matrix_json_path}" >&2
  exit 1
fi

m18_overall="$(jq -r '.m18Overall' "${kickoff_json_path}")"
primary_focus="$(jq -r '.primaryFocus' "${kickoff_json_path}")"
pending_gate_count="$(jq -r '.pendingGates | length' "${kickoff_json_path}")"
top_track="$(jq -r '.tracks[0].track' "${matrix_json_path}")"

selected_track="${top_track}"
slice_id=""
slice_title=""

if [ "${m18_overall}" != "PASS" ] || [ "${primary_focus}" = "stabilization" ] || [ "${pending_gate_count}" -gt 0 ]; then
  selected_track="runtime"
  slice_id="M19-S4-stabilization-remediation"
  slice_title="Resolve M18 carry-over risk before broad M19 expansion"
else
  case "${top_track}" in
    runtime)
      slice_id="M19-S4-runtime-hardening"
      slice_title="Advance runtime reliability and smoke confidence in M19"
      ;;
    release)
      slice_id="M19-S4-release-hardening"
      slice_title="Advance release/publish integrity for M19"
      ;;
    editor)
      slice_id="M19-S4-editor-expansion"
      slice_title="Advance editor productivity contracts in M19"
      ;;
    *)
      echo "unsupported M19 top priority track: ${top_track}" >&2
      exit 1
      ;;
  esac
fi

selector_json="$(
  jq -n \
    --arg version "0.1" \
    --arg kickoffJson "${kickoff_json_path}" \
    --arg matrixJson "${matrix_json_path}" \
    --arg m18Overall "${m18_overall}" \
    --arg primaryFocus "${primary_focus}" \
    --argjson pendingGateCount "${pending_gate_count}" \
    --arg selectedTrack "${selected_track}" \
    --arg sliceId "${slice_id}" \
    --arg sliceTitle "${slice_title}" \
    --arg closureGate "M19-C" \
    '{
      version: $version,
      inputs: {
        kickoffJson: $kickoffJson,
        matrixJson: $matrixJson
      },
      m18Overall: $m18Overall,
      primaryFocus: $primaryFocus,
      pendingGateCount: $pendingGateCount,
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

echo "M19 Next Slice Recommendation"
echo "m18Overall: ${m18_overall}"
echo "primaryFocus: ${primary_focus}"
echo "selectedTrack: ${selected_track}"
echo "sliceId: ${slice_id}"
echo "closureGate: M19-C"
echo "output: ${output_path}"
