#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/build-m37-alpha-release-notes.sh [--checklist-json <path>] [--output-json <path>] [--output-md <path>] [--format <markdown|json>]

Builds an M37-S7 alpha release-note package from the M37-S6 publish checklist delta artifact.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checklist_json="${root_dir}/build/m37-alpha-publish-checklist-delta.json"
output_json="${root_dir}/build/m37-alpha-release-notes.json"
output_md="${root_dir}/build/m37-alpha-release-notes.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --checklist-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      checklist_json="$2"
      shift 2
      ;;
    --checklist-json=*)
      checklist_json="${1#--checklist-json=}"
      shift
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_json="$2"
      shift 2
      ;;
    --output-json=*)
      output_json="${1#--output-json=}"
      shift
      ;;
    --output-md)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_md="$2"
      shift 2
      ;;
    --output-md=*)
      output_md="${1#--output-md=}"
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

if [ ! -f "${checklist_json}" ]; then
  echo "missing checklist json: ${checklist_json}" >&2
  exit 1
fi

if ! jq -e '.marker == "M37-S6" and .overall and .tagDecision and .identity.policyHash and .identity.compilerHash and .identity.runtimeHash and (.samples | type == "array")' "${checklist_json}" >/dev/null; then
  echo "invalid M37-S6 checklist contract: ${checklist_json}" >&2
  exit 1
fi

overall="$(jq -r '.overall' "${checklist_json}")"
tag_decision="$(jq -r '.tagDecision' "${checklist_json}")"
policy_hash="$(jq -r '.identity.policyHash' "${checklist_json}")"
compiler_hash="$(jq -r '.identity.compilerHash' "${checklist_json}")"
runtime_hash="$(jq -r '.identity.runtimeHash' "${checklist_json}")"
sample_count="$(jq -r '.samples | length' "${checklist_json}")"
sample_pending_count="$(jq -r '[.samples[] | select(.status != "PASS")] | length' "${checklist_json}")"

release_state="alpha-hold"
baseline_statement="No-stub alpha baseline is not ready yet."
next_action="Resolve pending checklist items and regenerate release notes."
if [ "${overall}" = "PASS" ] && [ "${tag_decision}" = "GO" ]; then
  release_state="alpha-ready"
  baseline_statement="No-stub alpha baseline is verified and ready for candidate tagging."
  next_action="Proceed with alpha tag decision execution and closure recording."
fi

known_limits_json="$(jq -n '[
  "TLS runtime support depends on OpenSSL toolchain availability; auto TLS mode falls back to non-TLS build when unavailable.",
  "v0.1-alpha scope is implementation-complete for supported paths but still intentionally narrow (single-runtime C backend baseline)."
]')"

notes_json="$(jq -n \
  --arg marker "M37-S7" \
  --arg sourceChecklist "${checklist_json}" \
  --arg releaseState "${release_state}" \
  --arg baselineStatement "${baseline_statement}" \
  --arg overall "${overall}" \
  --arg tagDecision "${tag_decision}" \
  --arg policyHash "${policy_hash}" \
  --arg compilerHash "${compiler_hash}" \
  --arg runtimeHash "${runtime_hash}" \
  --argjson sampleCount "${sample_count}" \
  --argjson samplePendingCount "${sample_pending_count}" \
  --argjson knownLimits "${known_limits_json}" \
  --arg nextAction "${next_action}" \
  '{
    version:"0.1",
    marker:$marker,
    sourceChecklist:$sourceChecklist,
    releaseState:$releaseState,
    baselineStatement:$baselineStatement,
    checklist:{overall:$overall,tagDecision:$tagDecision},
    identity:{policyHash:$policyHash,compilerHash:$compilerHash,runtimeHash:$runtimeHash},
    samples:{total:$sampleCount,pending:$samplePendingCount},
    knownLimits:$knownLimits,
    nextAction:$nextAction
  }')"

mkdir -p "$(dirname "${output_json}")" "$(dirname "${output_md}")"
printf '%s\n' "${notes_json}" > "${output_json}"

{
  echo "# M37 Alpha Release Notes Package"
  echo
  echo "- marker: \`M37-S7\`"
  echo "- source checklist: \`${checklist_json}\`"
  echo "- release state: \`${release_state}\`"
  echo
  echo "## No-Stub Baseline"
  echo
  echo "${baseline_statement}"
  echo
  echo "## Checklist Summary"
  echo
  echo "- overall: \`${overall}\`"
  echo "- tag decision: \`${tag_decision}\`"
  echo "- samples: \`${sample_count}\` total, \`${sample_pending_count}\` pending"
  echo
  echo "## Identity"
  echo
  echo "- policyHash: \`${policy_hash}\`"
  echo "- compilerHash: \`${compiler_hash}\`"
  echo "- runtimeHash: \`${runtime_hash}\`"
  echo
  echo "## Known Limits"
  echo
  jq -r '.[] | "- " + .' <<<"${known_limits_json}"
  echo
  echo "## Next Action"
  echo
  echo "${next_action}"
} > "${output_md}"

if [ "${output_format}" = "json" ]; then
  cat "${output_json}"
else
  cat "${output_md}"
fi
