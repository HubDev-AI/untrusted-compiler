#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--closure-json <path>] [--output <path>] [--format <markdown|json>]

Builds an M18 convergence summary across editor/release/runtime execution tracks.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
closure_json_path=""
output_path="${root_dir}/build/m18-track-convergence-summary.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --closure-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      closure_json_path="$2"
      shift 2
      ;;
    --closure-json=*)
      closure_json_path="${1#--closure-json=}"
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

closure_json=""
closure_source=""
if [ -n "${closure_json_path}" ]; then
  if [ ! -f "${closure_json_path}" ]; then
    echo "missing closure json file: ${closure_json_path}" >&2
    exit 1
  fi
  closure_json="$(cat "${closure_json_path}")"
  closure_source="${closure_json_path}"
else
  closure_json="$("${root_dir}/scripts/check-milestone-closure.sh" --format json)"
  closure_source="live:check-milestone-closure"
fi

if ! printf '%s\n' "${closure_json}" | jq -e '.overall and (.gates | type == "array")' >/dev/null; then
  echo "invalid closure json contract" >&2
  exit 1
fi

gate_status_for() {
  local gate="$1"
  local status
  status="$(printf '%s\n' "${closure_json}" | jq -r --arg gate "${gate}" '.gates[] | select(.gate == $gate) | .status' | head -n 1)"
  if [ -z "${status}" ] || [ "${status}" = "null" ]; then
    echo "missing required M18 gate in closure json: ${gate}" >&2
    exit 1
  fi
  echo "${status}"
}

editor_status="$(gate_status_for "M18-D")"
release_status="$(gate_status_for "M18-E")"
runtime_status="$(gate_status_for "M18-F")"

overall_status="PASS"
if [ "${editor_status}" != "PASS" ] || [ "${release_status}" != "PASS" ] || [ "${runtime_status}" != "PASS" ]; then
  overall_status="PENDING"
fi

next_action="Advance to post-M18 transition planning."
if [ "${overall_status}" != "PASS" ]; then
  next_action="Resolve pending M18 track gates before transition."
fi

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg source "${closure_source}" \
    --arg overall "${overall_status}" \
    --arg nextAction "${next_action}" \
    --arg editorStatus "${editor_status}" \
    --arg releaseStatus "${release_status}" \
    --arg runtimeStatus "${runtime_status}" \
    '{
      version: $version,
      source: $source,
      overall: $overall,
      tracks: [
        {track: "editor", gate: "M18-D", sliceId: "M18-S4-editor-contract-expansion", status: $editorStatus},
        {track: "release", gate: "M18-E", sliceId: "M18-S5-release-publish-integrity-contract-expansion", status: $releaseStatus},
        {track: "runtime", gate: "M18-F", sliceId: "M18-S6-runtime-confidence-hardening", status: $runtimeStatus}
      ],
      nextAction: $nextAction
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"
{
  echo "# M18 Track Convergence Summary"
  echo
  printf -- "- source: \`%s\`\n" "${closure_source}"
  printf -- "- overall: \`%s\`\n" "${overall_status}"
  echo
  echo "| Track | Gate | Slice | Status |"
  echo "| --- | --- | --- | --- |"
  jq -r '.tracks[] | "| \(.track) | \(.gate) | \(.sliceId) | \(.status) |"' <<<"${summary_json}"
  echo
  printf -- "- nextAction: %s\n" "${next_action}"
} > "${output_path}"

echo "m18 track convergence summary generated: ${output_path}"
