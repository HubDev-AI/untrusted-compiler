#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--matrix-json <path>] [--output <path>] [--format <text|json>]

Builds deterministic runtime-first de-stub execution plan for M33 from kickoff + matrix artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m33-kickoff-brief.json"
matrix_json_path="${root_dir}/build/m33-priority-matrix.json"
output_path="${root_dir}/build/m33-runtime-destub-plan.json"
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
  echo "missing M33 kickoff brief json: ${kickoff_json_path}" >&2
  exit 1
fi
if [ ! -f "${matrix_json_path}" ]; then
  echo "missing M33 priority matrix json: ${matrix_json_path}" >&2
  exit 1
fi

if ! jq -e '.m32Overall and .convergenceOverall and .primaryFocus and (.pendingGates | type == "array")' "${kickoff_json_path}" >/dev/null; then
  echo "invalid M33 kickoff brief json contract: ${kickoff_json_path}" >&2
  exit 1
fi
if ! jq -e '.tracks and (.tracks | type == "array") and (.tracks | length > 0)' "${matrix_json_path}" >/dev/null; then
  echo "invalid M33 priority matrix contract: ${matrix_json_path}" >&2
  exit 1
fi

m32_overall="$(jq -r '.m32Overall' "${kickoff_json_path}")"
convergence_overall="$(jq -r '.convergenceOverall' "${kickoff_json_path}")"
primary_focus="$(jq -r '.primaryFocus' "${kickoff_json_path}")"
pending_gate_count="$(jq -r '.pendingGates | length' "${kickoff_json_path}")"
top_track="$(jq -r '.tracks[0].track' "${matrix_json_path}")"

stabilization_mode=0
if [ "${m32_overall}" != "PASS" ] || [ "${convergence_overall}" != "PASS" ] || [ "${primary_focus}" = "stabilization" ] || [ "${pending_gate_count}" -gt 0 ]; then
  stabilization_mode=1
fi

selected_track="${top_track}"
if [ "${stabilization_mode}" -eq 1 ]; then
  selected_track="runtime"
fi

case "${selected_track}" in
  runtime|release|editor)
    ;;
  *)
    echo "unsupported M33 top priority track: ${selected_track}" >&2
    exit 1
    ;;
esac

plan_ids=""
plan_titles=""
plan_domains=""

if [ "${stabilization_mode}" -eq 1 ]; then
  plan_ids=$'M33-S4-runtime-validator-destub\nM33-S4-runtime-net-destub\nM33-S4-runtime-db-destub\nM33-S4-runtime-fs-destub\nM33-S4-runtime-secrets-destub'
  plan_titles=$'Lock validator/sanitizer runtime gates first\nHarden outbound/inbound network runtime paths\nReplace db runtime stubs with typed sink behavior\nReplace filesystem runtime stubs under PathSafe checks\nClose secrets runtime stubs and redaction/reveal boundaries'
  plan_domains=$'validators\nnet\ndb\nfs\nsecrets'
else
  case "${selected_track}" in
    runtime)
      plan_ids=$'M33-S4-runtime-db-destub\nM33-S4-runtime-net-destub\nM33-S4-runtime-validator-destub\nM33-S4-runtime-fs-destub\nM33-S4-runtime-secrets-destub'
      plan_titles=$'Replace db runtime stubs with typed sink behavior\nHarden outbound/inbound network runtime paths\nLock validator/sanitizer runtime gates\nReplace filesystem runtime stubs under PathSafe checks\nClose secrets runtime stubs and redaction/reveal boundaries'
      plan_domains=$'db\nnet\nvalidators\nfs\nsecrets'
      ;;
    release)
      plan_ids=$'M33-S4-runtime-db-destub\nM33-S4-runtime-validator-destub\nM33-S4-runtime-net-destub\nM33-S4-runtime-fs-destub\nM33-S4-runtime-secrets-destub'
      plan_titles=$'Replace db runtime stubs with typed sink behavior\nLock validator/sanitizer runtime gates\nHarden outbound/inbound network runtime paths\nReplace filesystem runtime stubs under PathSafe checks\nClose secrets runtime stubs and redaction/reveal boundaries'
      plan_domains=$'db\nvalidators\nnet\nfs\nsecrets'
      ;;
    editor)
      plan_ids=$'M33-S4-runtime-validator-destub\nM33-S4-runtime-db-destub\nM33-S4-runtime-net-destub\nM33-S4-runtime-fs-destub\nM33-S4-runtime-secrets-destub'
      plan_titles=$'Lock validator/sanitizer runtime gates first\nReplace db runtime stubs with typed sink behavior\nHarden outbound/inbound network runtime paths\nReplace filesystem runtime stubs under PathSafe checks\nClose secrets runtime stubs and redaction/reveal boundaries'
      plan_domains=$'validators\ndb\nnet\nfs\nsecrets'
      ;;
  esac
fi

plan_json="$(
  paste \
    <(printf '%s\n' "${plan_ids}") \
    <(printf '%s\n' "${plan_titles}") \
    <(printf '%s\n' "${plan_domains}") \
  | awk -F '\t' '
      NF == 3 {
        gsub(/"/, "\\\"", $2)
        printf("{\"id\":\"%s\",\"title\":\"%s\",\"domain\":\"%s\"}\n", $1, $2, $3)
      }
    ' \
  | jq -s '
      to_entries
      | map(.value + {order: (.key + 1)})
    '
)"

destub_json="$(
  jq -n \
    --arg version "0.1" \
    --arg kickoffJson "${kickoff_json_path}" \
    --arg matrixJson "${matrix_json_path}" \
    --arg m32Overall "${m32_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg primaryFocus "${primary_focus}" \
    --arg topTrack "${top_track}" \
    --arg selectedTrack "${selected_track}" \
    --argjson pendingGateCount "${pending_gate_count}" \
    --argjson stabilizationMode "${stabilization_mode}" \
    --argjson plan "${plan_json}" \
    --arg closureGate "M33-C" \
    '{
      version: $version,
      inputs: {
        kickoffJson: $kickoffJson,
        matrixJson: $matrixJson
      },
      m32Overall: $m32Overall,
      convergenceOverall: $convergenceOverall,
      primaryFocus: $primaryFocus,
      topTrack: $topTrack,
      selectedTrack: $selectedTrack,
      pendingGateCount: $pendingGateCount,
      stabilizationMode: ($stabilizationMode == 1),
      closureGate: $closureGate,
      plan: $plan,
      nextAction: ("Execute " + $plan[0].id + " and emit convergence evidence before broadening scope.")
    }'
)"

mkdir -p "$(dirname "${output_path}")"
printf '%s\n' "${destub_json}" > "${output_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${destub_json}"
  exit 0
fi

echo "M33 Runtime De-stub Plan"
echo "m32Overall: ${m32_overall}"
echo "convergenceOverall: ${convergence_overall}"
echo "primaryFocus: ${primary_focus}"
echo "topTrack: ${top_track}"
echo "selectedTrack: ${selected_track}"
echo "stabilizationMode: ${stabilization_mode}"
echo "closureGate: M33-C"
echo "firstPlanItem: $(jq -r '.plan[0].id' <<<"${destub_json}")"
echo "output: ${output_path}"
