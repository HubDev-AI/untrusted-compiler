#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--format <text|json>]

Prints the deterministic M17 operator troubleshooting matrix.
USAGE
}

output_format="text"

while [ "$#" -gt 0 ]; do
  case "$1" in
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

entries=(
  "artifact_missing|missing runtime-smoke artifact file:|runtime-smoke artifact validation|scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json|Re-run scripts/run-m17-operator-bootstrap.sh and inspect <tmp>/default and <tmp>/max-body."
  "runflags_shape_drift|run-metadata.txt runFlags field does not match expected runtime flag shape|runtime-smoke metadata shape gate|scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json|Ensure branch metadata maxBodyBytes and runFlags stay shape-aligned."
  "trace_header_missing|health.headers missing X-Trace-Id header|runtime trace-header contract|scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json|Confirm runtime responses include X-Trace-Id and re-run smoke."
  "trace_mismatch|users traceId mismatch between headers and body|runtime trace-correlation contract|scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json|Check response envelope traceId wiring against emitted headers."
  "handoff_chapter_token_missing|missing handoff chapter token:|M17 handoff readiness verifier|scripts/check-m17-operator-handoff-readiness.sh|Update docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md with canonical commands."
  "bundle_branch_missing|runtime-smoke bundle missing branch directory:|operator branch smoke completeness|scripts/run-m17-operator-bootstrap.sh|Ensure both default and max-body branch runs completed before bundle check."
  "closure_pending|overall: PENDING|strict closure audit|scripts/check-milestone-closure.sh --fail-on-pending|Resolve pending gates in closure output, then re-run handoff verification."
)

if [ "${output_format}" = "text" ]; then
  echo "M17 Operator Troubleshooting Matrix"
  echo
  printf '%-30s %-56s %-40s\n' "ID" "Diagnostic token" "Primary command"
  printf '%-30s %-56s %-40s\n' "--" "----------------" "---------------"
  for entry in "${entries[@]}"; do
    IFS='|' read -r id token _stage command _remediation <<<"${entry}"
    printf '%-30s %-56s %-40s\n' "${id}" "${token}" "${command}"
  done
  exit 0
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

entries_json='[]'
for entry in "${entries[@]}"; do
  IFS='|' read -r id token stage command remediation <<<"${entry}"
  entries_json="$(
    jq \
      --arg id "${id}" \
      --arg diagnosticToken "${token}" \
      --arg stage "${stage}" \
      --arg command "${command}" \
      --arg remediation "${remediation}" \
      '. + [{id: $id, diagnosticToken: $diagnosticToken, stage: $stage, command: $command, remediation: $remediation}]' \
      <<<"${entries_json}"
  )"
done

jq -n --arg version "0.1" --argjson entries "${entries_json}" '{version: $version, entries: $entries}'
