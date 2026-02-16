#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/build-m37-alpha-publish-checklist-delta.sh [--release-gate-dir <path>] [--output-json <path>] [--output-md <path>] [--format <markdown|json>]

Builds an M37-S6 alpha publish checklist delta artifact from release-alpha-gate evidence.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_gate_dir="${root_dir}/build/release-alpha-gate"
output_json="${root_dir}/build/m37-alpha-publish-checklist-delta.json"
output_md="${root_dir}/build/m37-alpha-publish-checklist-delta.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --release-gate-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      release_gate_dir="$2"
      shift 2
      ;;
    --release-gate-dir=*)
      release_gate_dir="${1#--release-gate-dir=}"
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

summary_path="${release_gate_dir}/summary.txt"
checksums_path="${release_gate_dir}/checksums.txt"
hello_meta_path="${release_gate_dir}/hello-build_metadata.json"
hello_api_meta_path="${release_gate_dir}/hello-api-build_metadata.json"
hello_audit_path="${release_gate_dir}/hello-audit.json"
hello_api_audit_path="${release_gate_dir}/hello-api-audit.json"

for required in \
  "${summary_path}" \
  "${checksums_path}" \
  "${hello_meta_path}" \
  "${hello_api_meta_path}" \
  "${hello_audit_path}" \
  "${hello_api_audit_path}"; do
  if [ ! -f "${required}" ]; then
    echo "missing required release artifact: ${required}" >&2
    exit 1
  fi
done

if ! jq -e '.policyHash and .compilerHash and .runtimeHash' "${hello_meta_path}" >/dev/null; then
  echo "invalid build metadata contract: ${hello_meta_path}" >&2
  exit 1
fi
if ! jq -e '.policyHash and .compilerHash and .runtimeHash' "${hello_api_meta_path}" >/dev/null; then
  echo "invalid build metadata contract: ${hello_api_meta_path}" >&2
  exit 1
fi
if ! jq -e '.policy.hash and .build.compilerHash and .build.runtimeHash and .summary.highestSeverity and .summary.riskScore' "${hello_audit_path}" >/dev/null; then
  echo "invalid audit report contract: ${hello_audit_path}" >&2
  exit 1
fi
if ! jq -e '.policy.hash and .build.compilerHash and .build.runtimeHash and .summary.highestSeverity and .summary.riskScore' "${hello_api_audit_path}" >/dev/null; then
  echo "invalid audit report contract: ${hello_api_audit_path}" >&2
  exit 1
fi

policy_identity_hash="$(awk '/^policy_identity_hash /{print $2}' "${checksums_path}" | head -n1)"
compiler_identity_hash="$(awk '/^compiler_identity_hash /{print $2}' "${checksums_path}" | head -n1)"
runtime_identity_hash="$(awk '/^runtime_identity_hash /{print $2}' "${checksums_path}" | head -n1)"

if [ -z "${policy_identity_hash}" ] || [ -z "${compiler_identity_hash}" ] || [ -z "${runtime_identity_hash}" ]; then
  echo "invalid release checksums contract: missing identity hashes in ${checksums_path}" >&2
  exit 1
fi

summary_release_gate_status="PENDING"
if rg -q '^sec4 alpha release gate: PASS$' "${summary_path}"; then
  summary_release_gate_status="PASS"
fi

summary_naming_lock_status="PENDING"
if rg -q '^naming lock: PASS$' "${summary_path}"; then
  summary_naming_lock_status="PASS"
fi

summary_closure_status="PENDING"
if rg -q '^milestone closure: PASS$' "${summary_path}"; then
  summary_closure_status="PASS"
fi

sample_status() {
  local sample="$1"
  local metadata_path="$2"
  local audit_path="$3"

  local meta_policy_hash
  local meta_compiler_hash
  local meta_runtime_hash
  local audit_policy_hash
  local audit_compiler_hash
  local audit_runtime_hash
  local highest_severity

  meta_policy_hash="$(jq -r '.policyHash' "${metadata_path}")"
  meta_compiler_hash="$(jq -r '.compilerHash' "${metadata_path}")"
  meta_runtime_hash="$(jq -r '.runtimeHash' "${metadata_path}")"
  audit_policy_hash="$(jq -r '.policy.hash' "${audit_path}")"
  audit_compiler_hash="$(jq -r '.build.compilerHash' "${audit_path}")"
  audit_runtime_hash="$(jq -r '.build.runtimeHash' "${audit_path}")"
  highest_severity="$(jq -r '.summary.highestSeverity' "${audit_path}")"

  local status="PASS"
  local evidence="identity hashes and audit severity pass"

  if [ "${meta_policy_hash}" != "${policy_identity_hash}" ] || [ "${audit_policy_hash}" != "${policy_identity_hash}" ]; then
    status="PENDING"
    evidence="policy identity hash mismatch"
  elif [ "${meta_compiler_hash}" != "${compiler_identity_hash}" ] || [ "${audit_compiler_hash}" != "${compiler_identity_hash}" ]; then
    status="PENDING"
    evidence="compiler identity hash mismatch"
  elif [ "${meta_runtime_hash}" != "${runtime_identity_hash}" ] || [ "${audit_runtime_hash}" != "${runtime_identity_hash}" ]; then
    status="PENDING"
    evidence="runtime identity hash mismatch"
  elif [ "${highest_severity}" = "HIGH" ] || [ "${highest_severity}" = "CRITICAL" ]; then
    status="PENDING"
    evidence="audit highest severity is ${highest_severity}"
  fi

  jq -n \
    --arg sample "${sample}" \
    --arg status "${status}" \
    --arg evidence "${evidence}" \
    --arg highestSeverity "${highest_severity}" \
    '{sample:$sample,status:$status,evidence:$evidence,highestSeverity:$highestSeverity}'
}

hello_sample_json="$(sample_status "hello" "${hello_meta_path}" "${hello_audit_path}")"
hello_api_sample_json="$(sample_status "hello-api" "${hello_api_meta_path}" "${hello_api_audit_path}")"

overall="PASS"
if [ "${summary_release_gate_status}" != "PASS" ] || \
   [ "${summary_naming_lock_status}" != "PASS" ] || \
   [ "${summary_closure_status}" != "PASS" ] || \
   [ "$(printf '%s\n' "${hello_sample_json}" | jq -r '.status')" != "PASS" ] || \
   [ "$(printf '%s\n' "${hello_api_sample_json}" | jq -r '.status')" != "PASS" ]; then
  overall="PENDING"
fi

tag_decision="HOLD"
next_action="Resolve pending checklist items, re-run release-alpha-gate, then rebuild this delta artifact."
if [ "${overall}" = "PASS" ]; then
  tag_decision="GO"
  next_action="Proceed with alpha tag decision and release-note publication."
fi

mkdir -p "$(dirname "${output_json}")" "$(dirname "${output_md}")"

json_payload="$(jq -n \
  --arg marker "M37-S6" \
  --arg releaseGateDir "${release_gate_dir}" \
  --arg policyIdentityHash "${policy_identity_hash}" \
  --arg compilerIdentityHash "${compiler_identity_hash}" \
  --arg runtimeIdentityHash "${runtime_identity_hash}" \
  --arg releaseGateStatus "${summary_release_gate_status}" \
  --arg namingLockStatus "${summary_naming_lock_status}" \
  --arg closureStatus "${summary_closure_status}" \
  --arg overall "${overall}" \
  --arg tagDecision "${tag_decision}" \
  --arg nextAction "${next_action}" \
  --argjson hello "${hello_sample_json}" \
  --argjson helloApi "${hello_api_sample_json}" \
  '{
    version:"0.1",
    marker:$marker,
    releaseGateDir:$releaseGateDir,
    identity:{
      policyHash:$policyIdentityHash,
      compilerHash:$compilerIdentityHash,
      runtimeHash:$runtimeIdentityHash
    },
    checks:[
      {id:"release_gate_pass",status:$releaseGateStatus,evidence:"summary.txt sec4 alpha release gate line"},
      {id:"naming_lock_pass",status:$namingLockStatus,evidence:"summary.txt naming lock line"},
      {id:"milestone_closure_pass",status:$closureStatus,evidence:"summary.txt milestone closure line"}
    ],
    samples:[$hello,$helloApi],
    overall:$overall,
    tagDecision:$tagDecision,
    nextAction:$nextAction
  }')"

printf '%s\n' "${json_payload}" > "${output_json}"

{
  echo "# M37 Alpha Publish Checklist Delta"
  echo
  echo "- marker: \`M37-S6\`"
  echo "- release gate dir: \`${release_gate_dir}\`"
  echo "- overall: \`${overall}\`"
  echo "- tag decision: \`${tag_decision}\`"
  echo
  echo "## Identity"
  echo
  echo "- policyHash: \`${policy_identity_hash}\`"
  echo "- compilerHash: \`${compiler_identity_hash}\`"
  echo "- runtimeHash: \`${runtime_identity_hash}\`"
  echo
  echo "## Core checks"
  echo
  echo "| check | status |"
  echo "| --- | --- |"
  echo "| release gate summary | ${summary_release_gate_status} |"
  echo "| naming lock summary | ${summary_naming_lock_status} |"
  echo "| milestone closure summary | ${summary_closure_status} |"
  echo
  echo "## Sample evidence"
  echo
  echo "| sample | status | highest severity | evidence |"
  echo "| --- | --- | --- | --- |"
  echo "| $(printf '%s\n' "${hello_sample_json}" | jq -r '.sample') | $(printf '%s\n' "${hello_sample_json}" | jq -r '.status') | $(printf '%s\n' "${hello_sample_json}" | jq -r '.highestSeverity') | $(printf '%s\n' "${hello_sample_json}" | jq -r '.evidence') |"
  echo "| $(printf '%s\n' "${hello_api_sample_json}" | jq -r '.sample') | $(printf '%s\n' "${hello_api_sample_json}" | jq -r '.status') | $(printf '%s\n' "${hello_api_sample_json}" | jq -r '.highestSeverity') | $(printf '%s\n' "${hello_api_sample_json}" | jq -r '.evidence') |"
  echo
  echo "## Next action"
  echo
  echo "${next_action}"
} > "${output_md}"

if [ "${output_format}" = "json" ]; then
  cat "${output_json}"
else
  cat "${output_md}"
fi
