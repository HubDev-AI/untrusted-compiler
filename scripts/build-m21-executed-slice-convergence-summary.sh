#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--selector-json <path>] [--runtime-execution-json <path>] [--output <path>] [--format <markdown|json>]

Builds a deterministic convergence summary for the first executed M21 slice.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
selector_json_path="${root_dir}/build/m21-next-slice.json"
runtime_execution_json_path="${root_dir}/build/m21-runtime-hardening.json"
output_path="${root_dir}/build/m21-executed-slice-convergence-summary.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
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

if [ ! -f "${selector_json_path}" ]; then
  echo "missing selector json: ${selector_json_path}" >&2
  exit 1
fi
if [ ! -f "${runtime_execution_json_path}" ]; then
  echo "missing runtime execution json: ${runtime_execution_json_path}" >&2
  exit 1
fi

if ! jq -e '.selectedTrack and .recommendation and .recommendation.id' "${selector_json_path}" >/dev/null; then
  echo "invalid selector contract: ${selector_json_path}" >&2
  exit 1
fi
if ! jq -e '.selectedTrack and .recommendationId and .status' "${runtime_execution_json_path}" >/dev/null; then
  echo "invalid runtime execution contract: ${runtime_execution_json_path}" >&2
  exit 1
fi

selector_track="$(jq -r '.selectedTrack' "${selector_json_path}")"
selector_recommendation_id="$(jq -r '.recommendation.id' "${selector_json_path}")"
selector_closure_gate="$(jq -r '.recommendation.closureGate // "M21-C"' "${selector_json_path}")"

runtime_track="$(jq -r '.selectedTrack' "${runtime_execution_json_path}")"
runtime_recommendation_id="$(jq -r '.recommendationId' "${runtime_execution_json_path}")"
runtime_status="$(jq -r '.status' "${runtime_execution_json_path}")"

if [ "${runtime_track}" != "${selector_track}" ]; then
  echo "runtime execution selectedTrack does not match selector track: ${runtime_track} != ${selector_track}" >&2
  exit 1
fi
if [ "${runtime_recommendation_id}" != "${selector_recommendation_id}" ]; then
  echo "runtime execution recommendationId does not match selector recommendation id: ${runtime_recommendation_id} != ${selector_recommendation_id}" >&2
  exit 1
fi

execution_pass="false"
if [ "${runtime_status}" = "PASS" ] || [ "${runtime_status}" = "DRY_RUN" ]; then
  execution_pass="true"
fi

overall="PENDING"
next_action="Resolve runtime hardening execution failures before continuing M21."
if [ "${execution_pass}" = "true" ]; then
  overall="PASS"
  next_action="Proceed with the next M21 executed slice."
fi

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg selectorJson "${selector_json_path}" \
    --arg runtimeExecutionJson "${runtime_execution_json_path}" \
    --arg selectedTrack "${selector_track}" \
    --arg recommendationId "${selector_recommendation_id}" \
    --arg closureGate "${selector_closure_gate}" \
    --arg runtimeStatus "${runtime_status}" \
    --arg overall "${overall}" \
    --arg nextAction "${next_action}" \
    --argjson executionPass "$( [ "${execution_pass}" = "true" ] && echo true || echo false )" \
    '{
      version: $version,
      inputs: {
        selectorJson: $selectorJson,
        runtimeExecutionJson: $runtimeExecutionJson
      },
      selectedTrack: $selectedTrack,
      recommendationId: $recommendationId,
      closureGate: $closureGate,
      runtimeStatus: $runtimeStatus,
      executionPass: $executionPass,
      overall: $overall,
      nextAction: $nextAction
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"
{
  echo "# M21 Executed Slice Convergence Summary"
  echo
  printf -- "- selectorJson: \`%s\`\n" "${selector_json_path}"
  printf -- "- runtimeExecutionJson: \`%s\`\n" "${runtime_execution_json_path}"
  printf -- "- selectedTrack: \`%s\`\n" "${selector_track}"
  printf -- "- recommendationId: \`%s\`\n" "${selector_recommendation_id}"
  printf -- "- closureGate: \`%s\`\n" "${selector_closure_gate}"
  printf -- "- runtimeStatus: \`%s\`\n" "${runtime_status}"
  printf -- "- executionPass: \`%s\`\n" "${execution_pass}"
  printf -- "- overall: \`%s\`\n" "${overall}"
  echo
  echo "## Next Action"
  printf -- "- %s\n" "${next_action}"
} > "${output_path}"

echo "m21 executed-slice convergence summary generated: ${output_path}"
