#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/build-m37-alpha-tag-decision-record.sh [--checklist-json <path>] [--release-notes-json <path>] [--decision <go|hold>] [--output-json <path>] [--output-md <path>] [--format <markdown|json>]

Builds an M37-S8 alpha tag-decision execution record from M37-S6 checklist + M37-S7 release notes artifacts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checklist_json="${root_dir}/build/m37-alpha-publish-checklist-delta.json"
release_notes_json="${root_dir}/build/m37-alpha-release-notes.json"
decision=""
output_json="${root_dir}/build/m37-alpha-tag-decision-record.json"
output_md="${root_dir}/build/m37-alpha-tag-decision-record.md"
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
    --release-notes-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      release_notes_json="$2"
      shift 2
      ;;
    --release-notes-json=*)
      release_notes_json="${1#--release-notes-json=}"
      shift
      ;;
    --decision)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      decision="$2"
      shift 2
      ;;
    --decision=*)
      decision="${1#--decision=}"
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
if [ ! -f "${release_notes_json}" ]; then
  echo "missing release-notes json: ${release_notes_json}" >&2
  exit 1
fi

if ! jq -e '.marker == "M37-S6" and .overall and .tagDecision and .identity.policyHash and .identity.compilerHash and .identity.runtimeHash' "${checklist_json}" >/dev/null; then
  echo "invalid M37-S6 checklist contract: ${checklist_json}" >&2
  exit 1
fi
if ! jq -e '.marker == "M37-S7" and .releaseState and .checklist.overall and .checklist.tagDecision and .identity.policyHash and .identity.compilerHash and .identity.runtimeHash' "${release_notes_json}" >/dev/null; then
  echo "invalid M37-S7 release-notes contract: ${release_notes_json}" >&2
  exit 1
fi

checklist_overall="$(jq -r '.overall' "${checklist_json}")"
checklist_tag_decision="$(jq -r '.tagDecision' "${checklist_json}")"
release_state="$(jq -r '.releaseState' "${release_notes_json}")"
notes_tag_decision="$(jq -r '.checklist.tagDecision' "${release_notes_json}")"

policy_hash="$(jq -r '.identity.policyHash' "${checklist_json}")"
compiler_hash="$(jq -r '.identity.compilerHash' "${checklist_json}")"
runtime_hash="$(jq -r '.identity.runtimeHash' "${checklist_json}")"

if [ -z "${decision}" ]; then
  decision="${notes_tag_decision}"
fi

decision_normalized="$(printf '%s' "${decision}" | tr '[:upper:]' '[:lower:]')"
case "${decision_normalized}" in
  go|hold)
    ;;
  *)
    echo "invalid decision: ${decision} (expected go|hold)" >&2
    exit 2
    ;;
esac

if [ "${decision_normalized}" = "go" ]; then
  if [ "${checklist_overall}" != "PASS" ] || [ "${checklist_tag_decision}" != "GO" ] || [ "${release_state}" != "alpha-ready" ] || [ "${notes_tag_decision}" != "GO" ]; then
    echo "cannot execute GO decision: checklist/release-notes prerequisites are not PASS/GO" >&2
    exit 1
  fi
fi

if [ "$(jq -r '.identity.policyHash' "${release_notes_json}")" != "${policy_hash}" ] || \
   [ "$(jq -r '.identity.compilerHash' "${release_notes_json}")" != "${compiler_hash}" ] || \
   [ "$(jq -r '.identity.runtimeHash' "${release_notes_json}")" != "${runtime_hash}" ]; then
  echo "identity mismatch between checklist and release-notes artifacts" >&2
  exit 1
fi

decision_executed="$(printf '%s' "${decision_normalized}" | tr '[:lower:]' '[:upper:]')"
closure_outcome="PENDING"
closure_reason="alpha tag is on hold"
if [ "${decision_executed}" = "GO" ]; then
  closure_outcome="PASS"
  closure_reason="alpha candidate tagging approved by checklist and release-note package"
fi

next_action="keep hold state and resolve pending release readiness issues"
if [ "${closure_outcome}" = "PASS" ]; then
  next_action="create alpha tag and begin post-tag verification"
fi

record_json="$(jq -n \
  --arg marker "M37-S8" \
  --arg checklistJson "${checklist_json}" \
  --arg releaseNotesJson "${release_notes_json}" \
  --arg decisionRequested "${decision_executed}" \
  --arg decisionExecuted "${decision_executed}" \
  --arg closureOutcome "${closure_outcome}" \
  --arg closureReason "${closure_reason}" \
  --arg policyHash "${policy_hash}" \
  --arg compilerHash "${compiler_hash}" \
  --arg runtimeHash "${runtime_hash}" \
  --arg nextAction "${next_action}" \
  '{
    version:"0.1",
    marker:$marker,
    sources:{checklist:$checklistJson,releaseNotes:$releaseNotesJson},
    decision:{requested:$decisionRequested,executed:$decisionExecuted},
    closure:{outcome:$closureOutcome,reason:$closureReason},
    identity:{policyHash:$policyHash,compilerHash:$compilerHash,runtimeHash:$runtimeHash},
    nextAction:$nextAction
  }')"

mkdir -p "$(dirname "${output_json}")" "$(dirname "${output_md}")"
printf '%s\n' "${record_json}" > "${output_json}"

{
  echo "# M37 Alpha Tag Decision Record"
  echo
  echo "- marker: \`M37-S8\`"
  echo "- checklist artifact: \`${checklist_json}\`"
  echo "- release-notes artifact: \`${release_notes_json}\`"
  echo
  echo "## Decision"
  echo
  echo "- requested: \`${decision_executed}\`"
  echo "- executed: \`${decision_executed}\`"
  echo
  echo "## Closure"
  echo
  echo "- outcome: \`${closure_outcome}\`"
  echo "- reason: ${closure_reason}"
  echo
  echo "## Identity"
  echo
  echo "- policyHash: \`${policy_hash}\`"
  echo "- compilerHash: \`${compiler_hash}\`"
  echo "- runtimeHash: \`${runtime_hash}\`"
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
