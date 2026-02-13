#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_DIR="${ROOT_DIR}/build/release-alpha-gate"

usage() {
  cat <<'USAGE'
usage: scripts/verify-release-promotion-inputs.sh [--artifacts-dir <path>]

Validates that alpha release-gate artifacts are internally consistent for
promotion workflows (checksums, identity hashes, build metadata, audit reports).
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifacts-dir)
      ARTIFACTS_DIR="$2"
      shift 2
      ;;
    --artifacts-dir=*)
      ARTIFACTS_DIR="${1#--artifacts-dir=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

hash_file() {
  local file_path="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file_path}" | awk '{print $1}'
    return
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print $1}'
    return
  fi
  echo "error: no sha256 command found (need shasum or sha256sum)" >&2
  exit 2
}

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "error: required artifact file missing: ${path}" >&2
    exit 1
  fi
}

require_jq() {
  if command -v jq >/dev/null 2>&1; then
    return
  fi
  echo "error: jq is required for release artifact verification" >&2
  exit 2
}

read_json_field() {
  local file_path="$1"
  local filter="$2"
  local value
  value="$(jq -r "${filter} // empty" "${file_path}")"
  if [[ -z "${value}" ]]; then
    echo "error: missing JSON field '${filter}' in ${file_path}" >&2
    exit 1
  fi
  echo "${value}"
}

read_checksum_value() {
  local checksums_path="$1"
  local key="$2"
  local value
  value="$(awk -v k="${key}" '$1 == k { print $2 }' "${checksums_path}" | tail -n 1)"
  if [[ -z "${value}" ]]; then
    echo "error: missing checksum entry for key '${key}' in ${checksums_path}" >&2
    exit 1
  fi
  echo "${value}"
}

read_sample_checksum_value() {
  local checksums_path="$1"
  local sample="$2"
  local key="$3"
  local value
  value="$(awk -v s="${sample}" -v k="${key}" '$1 == s && $2 == k { print $3 }' "${checksums_path}" | tail -n 1)"
  if [[ -z "${value}" ]]; then
    echo "error: missing sample checksum entry for sample='${sample}' key='${key}' in ${checksums_path}" >&2
    exit 1
  fi
  echo "${value}"
}

read_summary_value() {
  local summary_path="$1"
  local label="$2"
  local value
  value="$(grep -E "^${label}: " "${summary_path}" | sed -E "s/^${label}: //" | tail -n 1)"
  if [[ -z "${value}" ]]; then
    echo "error: missing summary entry '${label}' in ${summary_path}" >&2
    exit 1
  fi
  echo "${value}"
}

require_jq

CHECKSUMS_PATH="${ARTIFACTS_DIR}/checksums.txt"
SUMMARY_PATH="${ARTIFACTS_DIR}/summary.txt"
RUNTIME_HEADER_PATH="${ARTIFACTS_DIR}/sec4_runtime.h"
RUNTIME_SOURCE_PATH="${ARTIFACTS_DIR}/sec4_runtime.c"

require_file "${CHECKSUMS_PATH}"
require_file "${SUMMARY_PATH}"
require_file "${RUNTIME_HEADER_PATH}"
require_file "${RUNTIME_SOURCE_PATH}"

policy_profile_path="$(find "${ARTIFACTS_DIR}" -maxdepth 1 -type f -name '*.sec4.policy' | head -n 1)"
if [[ -z "${policy_profile_path}" ]]; then
  echo "error: missing copied policy profile in ${ARTIFACTS_DIR}" >&2
  exit 1
fi

policy_profile_sha_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "policy_profile_sha256")"
runtime_header_sha_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "runtime_header_sha256")"
runtime_source_sha_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "runtime_source_sha256")"
policy_identity_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "policy_identity_hash")"
compiler_identity_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "compiler_identity_hash")"
runtime_identity_expected="$(read_checksum_value "${CHECKSUMS_PATH}" "runtime_identity_hash")"

policy_profile_sha_actual="$(hash_file "${policy_profile_path}")"
runtime_header_sha_actual="$(hash_file "${RUNTIME_HEADER_PATH}")"
runtime_source_sha_actual="$(hash_file "${RUNTIME_SOURCE_PATH}")"

if [[ "${policy_profile_sha_actual}" != "${policy_profile_sha_expected}" ]]; then
  echo "error: policy profile sha mismatch in artifacts" >&2
  exit 1
fi
if [[ "${runtime_header_sha_actual}" != "${runtime_header_sha_expected}" ]]; then
  echo "error: runtime header sha mismatch in artifacts" >&2
  exit 1
fi
if [[ "${runtime_source_sha_actual}" != "${runtime_source_sha_expected}" ]]; then
  echo "error: runtime source sha mismatch in artifacts" >&2
  exit 1
fi

summary_policy_identity="$(read_summary_value "${SUMMARY_PATH}" "policy identity hash")"
summary_compiler_identity="$(read_summary_value "${SUMMARY_PATH}" "compiler identity hash")"
summary_runtime_identity="$(read_summary_value "${SUMMARY_PATH}" "runtime identity hash")"
summary_naming_lock="$(read_summary_value "${SUMMARY_PATH}" "naming lock")"

if [[ "${summary_policy_identity}" != "${policy_identity_expected}" ]]; then
  echo "error: summary/checksums mismatch for policy identity hash" >&2
  exit 1
fi
if [[ "${summary_compiler_identity}" != "${compiler_identity_expected}" ]]; then
  echo "error: summary/checksums mismatch for compiler identity hash" >&2
  exit 1
fi
if [[ "${summary_runtime_identity}" != "${runtime_identity_expected}" ]]; then
  echo "error: summary/checksums mismatch for runtime identity hash" >&2
  exit 1
fi
if [[ "${summary_naming_lock}" != "PASS" ]]; then
  echo "error: summary naming lock status must be PASS" >&2
  exit 1
fi

sample_count=0
for meta_path in "${ARTIFACTS_DIR}"/*-build_metadata.json; do
  [[ -e "${meta_path}" ]] || continue
  sample="$(basename "${meta_path}" -build_metadata.json)"
  sample_count=$((sample_count + 1))

  sbom_path="${ARTIFACTS_DIR}/${sample}-sbom.json"
  audit_path="${ARTIFACTS_DIR}/${sample}-audit.json"
  require_file "${sbom_path}"
  require_file "${audit_path}"

  meta_sha_expected="$(read_sample_checksum_value "${CHECKSUMS_PATH}" "${sample}" "build_metadata_sha256")"
  sbom_sha_expected="$(read_sample_checksum_value "${CHECKSUMS_PATH}" "${sample}" "sbom_sha256")"
  meta_sha_actual="$(hash_file "${meta_path}")"
  sbom_sha_actual="$(hash_file "${sbom_path}")"

  if [[ "${meta_sha_actual}" != "${meta_sha_expected}" ]]; then
    echo "error: build metadata sha mismatch for sample '${sample}'" >&2
    exit 1
  fi
  if [[ "${sbom_sha_actual}" != "${sbom_sha_expected}" ]]; then
    echo "error: sbom sha mismatch for sample '${sample}'" >&2
    exit 1
  fi

  meta_policy="$(read_json_field "${meta_path}" '.policyHash')"
  meta_compiler="$(read_json_field "${meta_path}" '.compilerHash')"
  meta_runtime="$(read_json_field "${meta_path}" '.runtimeHash')"
  audit_policy="$(read_json_field "${audit_path}" '.policy.hash')"
  audit_compiler="$(read_json_field "${audit_path}" '.build.compilerHash')"
  audit_runtime="$(read_json_field "${audit_path}" '.build.runtimeHash')"

  if [[ "${meta_policy}" != "${policy_identity_expected}" || "${audit_policy}" != "${policy_identity_expected}" ]]; then
    echo "error: policy identity mismatch for sample '${sample}'" >&2
    exit 1
  fi
  if [[ "${meta_compiler}" != "${compiler_identity_expected}" || "${audit_compiler}" != "${compiler_identity_expected}" ]]; then
    echo "error: compiler identity mismatch for sample '${sample}'" >&2
    exit 1
  fi
  if [[ "${meta_runtime}" != "${runtime_identity_expected}" || "${audit_runtime}" != "${runtime_identity_expected}" ]]; then
    echo "error: runtime identity mismatch for sample '${sample}'" >&2
    exit 1
  fi
done

if [[ "${sample_count}" -eq 0 ]]; then
  echo "error: no sample build metadata artifacts found in ${ARTIFACTS_DIR}" >&2
  exit 1
fi

echo "ok: release promotion inputs verified (${sample_count} samples)"
