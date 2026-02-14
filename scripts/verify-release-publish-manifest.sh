#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST_PATH="${ROOT_DIR}/build/release-alpha-gate/publish-manifest.json"
ARTIFACTS_DIR=""

usage() {
  cat <<'USAGE'
usage: scripts/verify-release-publish-manifest.sh [--manifest <path>] [--artifacts-dir <path>]

Validates publish-manifest.json against release artifact files/checksum entries.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest)
      MANIFEST_PATH="$2"
      shift 2
      ;;
    --manifest=*)
      MANIFEST_PATH="${1#--manifest=}"
      shift
      ;;
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

if [[ -z "${ARTIFACTS_DIR}" ]]; then
  ARTIFACTS_DIR="$(cd "$(dirname "${MANIFEST_PATH}")" && pwd)"
fi

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "error: required file missing: ${path}" >&2
    exit 1
  fi
}

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

assert_hash_matches() {
  local file_path="$1"
  local expected_sha="$2"
  local label="$3"
  local actual_sha
  actual_sha="$(hash_file "${file_path}")"
  if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "error: artifact checksum mismatch for ${label}" >&2
    exit 1
  fi
}

read_manifest_field() {
  local filter="$1"
  local value
  value="$(jq -r "${filter} // empty" "${MANIFEST_PATH}")"
  if [[ -z "${value}" ]]; then
    echo "error: missing manifest field '${filter}' in ${MANIFEST_PATH}" >&2
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
    echo "error: missing checksum entry '${key}' in ${checksums_path}" >&2
    exit 1
  fi
  echo "${value}"
}

read_summary_value() {
  local summary_path="$1"
  local key="$2"
  local value
  value="$(awk -F ': ' -v k="${key}" '$1 == k { print $2 }' "${summary_path}" | tail -n 1)"
  if [[ -z "${value}" ]]; then
    echo "error: missing summary entry '${key}' in ${summary_path}" >&2
    exit 1
  fi
  echo "${value}"
}

read_sample_checksum() {
  local checksums_path="$1"
  local sample="$2"
  local key="$3"
  local value
  value="$(awk -v s="${sample}" -v k="${key}" '$1 == s && $2 == k { print $3 }' "${checksums_path}" | tail -n 1)"
  if [[ -z "${value}" ]]; then
    echo "error: missing sample checksum entry sample='${sample}' key='${key}'" >&2
    exit 1
  fi
  echo "${value}"
}

require_file "${MANIFEST_PATH}"

if ! jq -e '.version == "0.1" and .tool == "sec4"' "${MANIFEST_PATH}" >/dev/null; then
  echo "error: manifest version/tool mismatch in ${MANIFEST_PATH}" >&2
  exit 1
fi

checksums_file="$(read_manifest_field '.artifacts.checksums')"
summary_file="$(read_manifest_field '.artifacts.summary')"
policy_file="$(read_manifest_field '.artifacts.policyFile')"
runtime_header_file="$(read_manifest_field '.artifacts.runtimeHeader')"
runtime_source_file="$(read_manifest_field '.artifacts.runtimeSource')"

checksums_path="${ARTIFACTS_DIR}/${checksums_file}"
summary_path="${ARTIFACTS_DIR}/${summary_file}"
policy_path="${ARTIFACTS_DIR}/${policy_file}"
runtime_header_path="${ARTIFACTS_DIR}/${runtime_header_file}"
runtime_source_path="${ARTIFACTS_DIR}/${runtime_source_file}"

require_file "${checksums_path}"
require_file "${summary_path}"
require_file "${policy_path}"
require_file "${runtime_header_path}"
require_file "${runtime_source_path}"

manifest_policy_profile_sha="$(read_manifest_field '.identity.policyProfileSha256')"
manifest_policy_identity="$(read_manifest_field '.identity.policyIdentityHash')"
manifest_compiler_identity="$(read_manifest_field '.identity.compilerIdentityHash')"
manifest_runtime_identity="$(read_manifest_field '.identity.runtimeIdentityHash')"
manifest_check_naming_lock="$(read_manifest_field '.checks.namingLock')"
manifest_check_milestone_closure="$(read_manifest_field '.checks.milestoneClosure')"

checksum_policy_profile_sha="$(read_checksum_value "${checksums_path}" "policy_profile_sha256")"
checksum_policy_identity="$(read_checksum_value "${checksums_path}" "policy_identity_hash")"
checksum_compiler_identity="$(read_checksum_value "${checksums_path}" "compiler_identity_hash")"
checksum_runtime_identity="$(read_checksum_value "${checksums_path}" "runtime_identity_hash")"
checksum_runtime_header_sha="$(read_checksum_value "${checksums_path}" "runtime_header_sha256")"
checksum_runtime_source_sha="$(read_checksum_value "${checksums_path}" "runtime_source_sha256")"
summary_naming_lock="$(read_summary_value "${summary_path}" "naming lock")"
summary_milestone_closure="$(read_summary_value "${summary_path}" "milestone closure")"

if [[ "${manifest_policy_profile_sha}" != "${checksum_policy_profile_sha}" ]]; then
  echo "error: manifest/checksums mismatch for policyProfileSha256" >&2
  exit 1
fi
if [[ "${manifest_policy_identity}" != "${checksum_policy_identity}" ]]; then
  echo "error: manifest/checksums mismatch for policyIdentityHash" >&2
  exit 1
fi
if [[ "${manifest_compiler_identity}" != "${checksum_compiler_identity}" ]]; then
  echo "error: manifest/checksums mismatch for compilerIdentityHash" >&2
  exit 1
fi
if [[ "${manifest_runtime_identity}" != "${checksum_runtime_identity}" ]]; then
  echo "error: manifest/checksums mismatch for runtimeIdentityHash" >&2
  exit 1
fi
if [[ "${manifest_check_naming_lock}" != "${summary_naming_lock}" ]]; then
  echo "error: manifest/summary mismatch for checks.namingLock" >&2
  exit 1
fi
if [[ "${manifest_check_milestone_closure}" != "${summary_milestone_closure}" ]]; then
  echo "error: manifest/summary mismatch for checks.milestoneClosure" >&2
  exit 1
fi
if [[ "${manifest_check_naming_lock}" != "PASS" ]]; then
  echo "error: manifest checks.namingLock must be PASS" >&2
  exit 1
fi
if [[ "${manifest_check_milestone_closure}" != "PASS" ]]; then
  echo "error: manifest checks.milestoneClosure must be PASS" >&2
  exit 1
fi

assert_hash_matches "${policy_path}" "${checksum_policy_profile_sha}" "policy profile copy"
assert_hash_matches "${runtime_header_path}" "${checksum_runtime_header_sha}" "runtime header copy"
assert_hash_matches "${runtime_source_path}" "${checksum_runtime_source_sha}" "runtime source copy"

sample_count="$(jq -r '.artifacts.samples | length' "${MANIFEST_PATH}")"
if [[ "${sample_count}" -eq 0 ]]; then
  echo "error: manifest contains no sample artifacts" >&2
  exit 1
fi

for idx in $(seq 0 $((sample_count - 1))); do
  sample_name="$(jq -r ".artifacts.samples[${idx}].name // empty" "${MANIFEST_PATH}")"
  build_meta_file="$(jq -r ".artifacts.samples[${idx}].buildMetadata // empty" "${MANIFEST_PATH}")"
  sbom_file="$(jq -r ".artifacts.samples[${idx}].sbom // empty" "${MANIFEST_PATH}")"
  audit_file="$(jq -r ".artifacts.samples[${idx}].audit // empty" "${MANIFEST_PATH}")"
  map_file="$(jq -r ".artifacts.samples[${idx}].securityMap // empty" "${MANIFEST_PATH}")"

  if [[ -z "${sample_name}" || -z "${build_meta_file}" || -z "${sbom_file}" || -z "${audit_file}" || -z "${map_file}" ]]; then
    echo "error: manifest sample entry ${idx} is incomplete" >&2
    exit 1
  fi

  require_file "${ARTIFACTS_DIR}/${build_meta_file}"
  require_file "${ARTIFACTS_DIR}/${sbom_file}"
  require_file "${ARTIFACTS_DIR}/${audit_file}"
  require_file "${ARTIFACTS_DIR}/${map_file}"

  expected_build_meta_sha="$(read_sample_checksum "${checksums_path}" "${sample_name}" "build_metadata_sha256")"
  expected_sbom_sha="$(read_sample_checksum "${checksums_path}" "${sample_name}" "sbom_sha256")"

  assert_hash_matches "${ARTIFACTS_DIR}/${build_meta_file}" "${expected_build_meta_sha}" "sample '${sample_name}' build metadata"
  assert_hash_matches "${ARTIFACTS_DIR}/${sbom_file}" "${expected_sbom_sha}" "sample '${sample_name}' sbom"
done

echo "ok: publish manifest verified (${sample_count} samples)"
