#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--selector-json <path>] [--runtime-smoke-root <path>] [--runtime-smoke-index <path>] [--format <text|json>] [--dry-run]

Executes the first runtime hardening slice selected by the M24 selector artifact.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
selector_json_path="${root_dir}/build/m24-next-slice.json"
runtime_smoke_root="${root_dir}/build/runtime-smoke"
runtime_smoke_index="${root_dir}/build/runtime-smoke/runtime-smoke-branch-index.json"
output_format="text"
dry_run="false"

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
    --runtime-smoke-root)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      runtime_smoke_root="$2"
      shift 2
      ;;
    --runtime-smoke-root=*)
      runtime_smoke_root="${1#--runtime-smoke-root=}"
      shift
      ;;
    --runtime-smoke-index)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      runtime_smoke_index="$2"
      shift 2
      ;;
    --runtime-smoke-index=*)
      runtime_smoke_index="${1#--runtime-smoke-index=}"
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
    --dry-run)
      dry_run="true"
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

if [ ! -f "${selector_json_path}" ]; then
  echo "missing selector json: ${selector_json_path}" >&2
  exit 1
fi

if ! jq -e '.selectedTrack and .recommendation and .recommendation.id' "${selector_json_path}" >/dev/null; then
  echo "invalid selector contract: ${selector_json_path}" >&2
  exit 1
fi

selected_track="$(jq -r '.selectedTrack' "${selector_json_path}")"
recommendation_id="$(jq -r '.recommendation.id' "${selector_json_path}")"
closure_gate="$(jq -r '.recommendation.closureGate // "M24-C"' "${selector_json_path}")"

if [ "${selected_track}" != "runtime" ]; then
  echo "selector did not choose runtime track: ${selected_track}" >&2
  exit 1
fi

if ! [[ "${recommendation_id}" =~ ^M24-S4-runtime- ]]; then
  echo "selector runtime recommendation id is not an M24-S4 runtime slice: ${recommendation_id}" >&2
  exit 1
fi

commands=(
  "scripts/check-runtime-smoke-bundle.sh --artifacts-root ${runtime_smoke_root} --index-path ${runtime_smoke_index}"
  "scripts/test-m16-runtime-http-coverage.sh"
  "scripts/check-milestone-closure.sh --fail-on-pending"
)

commands_json='[]'
for command in "${commands[@]}"; do
  commands_json="$(jq -c --arg command "${command}" '. + [$command]' <<<"${commands_json}")"
done

if [ "${dry_run}" = "true" ]; then
  status="DRY_RUN"
else
  for command in "${commands[@]}"; do
    (cd "${root_dir}" && bash -lc "${command}")
  done
  status="PASS"
fi

result_json="$(
  jq -n \
    --arg version "0.1" \
    --arg selectorJson "${selector_json_path}" \
    --arg selectedTrack "${selected_track}" \
    --arg recommendationId "${recommendation_id}" \
    --arg closureGate "${closure_gate}" \
    --arg runtimeSmokeRoot "${runtime_smoke_root}" \
    --arg runtimeSmokeIndex "${runtime_smoke_index}" \
    --arg status "${status}" \
    --argjson dryRun "$( [ "${dry_run}" = "true" ] && echo true || echo false )" \
    --argjson commands "${commands_json}" \
    '{
      version: $version,
      selectorJson: $selectorJson,
      selectedTrack: $selectedTrack,
      recommendationId: $recommendationId,
      closureGate: $closureGate,
      runtimeSmoke: {
        artifactsRoot: $runtimeSmokeRoot,
        indexPath: $runtimeSmokeIndex
      },
      commands: $commands,
      dryRun: $dryRun,
      status: $status
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${result_json}"
  exit 0
fi

echo "M24 Runtime Hardening Execution"
echo "selectorJson: ${selector_json_path}"
echo "selectedTrack: ${selected_track}"
echo "recommendationId: ${recommendation_id}"
echo "closureGate: ${closure_gate}"
echo "runtimeSmokeRoot: ${runtime_smoke_root}"
echo "runtimeSmokeIndex: ${runtime_smoke_index}"
echo "status: ${status}"

if [ "${dry_run}" = "true" ]; then
  echo "commands:"
  for command in "${commands[@]}"; do
    echo "  - ${command}"
  done
fi
