#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_DIR="${ROOT_DIR}/build/release-alpha-gate"
OUT_PATH=""

usage() {
  cat <<'USAGE'
usage: scripts/generate-release-publish-manifest.sh [--artifacts-dir <path>] [--out <path>]

Generates publish-manifest.json from verified release-gate artifacts for
promotion pipeline consumption.
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
    --out)
      OUT_PATH="$2"
      shift 2
      ;;
    --out=*)
      OUT_PATH="${1#--out=}"
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

if [[ -z "${OUT_PATH}" ]]; then
  OUT_PATH="${ARTIFACTS_DIR}/publish-manifest.json"
fi

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "error: required file missing: ${path}" >&2
    exit 1
  fi
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

CHECKSUMS_PATH="${ARTIFACTS_DIR}/checksums.txt"
SUMMARY_PATH="${ARTIFACTS_DIR}/summary.txt"
RUNTIME_HEADER_PATH="${ARTIFACTS_DIR}/sec4_runtime.h"
RUNTIME_SOURCE_PATH="${ARTIFACTS_DIR}/sec4_runtime.c"

require_file "${CHECKSUMS_PATH}"
require_file "${SUMMARY_PATH}"
require_file "${RUNTIME_HEADER_PATH}"
require_file "${RUNTIME_SOURCE_PATH}"

policy_file="$(find "${ARTIFACTS_DIR}" -maxdepth 1 -type f -name '*.sec4.policy' | head -n 1)"
if [[ -z "${policy_file}" ]]; then
  echo "error: policy profile copy not found in ${ARTIFACTS_DIR}" >&2
  exit 1
fi

policy_profile_sha256="$(read_checksum_value "${CHECKSUMS_PATH}" "policy_profile_sha256")"
policy_identity_hash="$(read_checksum_value "${CHECKSUMS_PATH}" "policy_identity_hash")"
compiler_identity_hash="$(read_checksum_value "${CHECKSUMS_PATH}" "compiler_identity_hash")"
runtime_identity_hash="$(read_checksum_value "${CHECKSUMS_PATH}" "runtime_identity_hash")"

sample_names=()
while IFS= read -r sample_name; do
  [[ -z "${sample_name}" ]] && continue
  sample_names+=("${sample_name}")
done < <(find "${ARTIFACTS_DIR}" -maxdepth 1 -type f -name '*-build_metadata.json' -print \
  | sed -E 's#.*/([^/]+)-build_metadata\.json#\1#' \
  | sort)

if [[ "${#sample_names[@]}" -eq 0 ]]; then
  echo "error: no sample build metadata artifacts found in ${ARTIFACTS_DIR}" >&2
  exit 1
fi

samples_json='[]'
for sample in "${sample_names[@]}"; do
  build_meta_file="${sample}-build_metadata.json"
  sbom_file="${sample}-sbom.json"
  audit_file="${sample}-audit.json"
  security_map_file="${sample}-security_map.json"

  require_file "${ARTIFACTS_DIR}/${build_meta_file}"
  require_file "${ARTIFACTS_DIR}/${sbom_file}"
  require_file "${ARTIFACTS_DIR}/${audit_file}"
  require_file "${ARTIFACTS_DIR}/${security_map_file}"

  samples_json="$(jq -c \
    --arg name "${sample}" \
    --arg buildMetadata "${build_meta_file}" \
    --arg sbom "${sbom_file}" \
    --arg audit "${audit_file}" \
    --arg securityMap "${security_map_file}" \
    '. + [{
      name: $name,
      buildMetadata: $buildMetadata,
      sbom: $sbom,
      audit: $audit,
      securityMap: $securityMap
    }]' <<<"${samples_json}")"
done

generated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
out_dir="$(dirname "${OUT_PATH}")"
mkdir -p "${out_dir}"

jq -n \
  --arg version "0.1" \
  --arg generatedAt "${generated_at}" \
  --arg policyFile "$(basename "${policy_file}")" \
  --arg checksums "$(basename "${CHECKSUMS_PATH}")" \
  --arg summary "$(basename "${SUMMARY_PATH}")" \
  --arg runtimeHeader "$(basename "${RUNTIME_HEADER_PATH}")" \
  --arg runtimeSource "$(basename "${RUNTIME_SOURCE_PATH}")" \
  --arg policyProfileSha256 "${policy_profile_sha256}" \
  --arg policyIdentityHash "${policy_identity_hash}" \
  --arg compilerIdentityHash "${compiler_identity_hash}" \
  --arg runtimeIdentityHash "${runtime_identity_hash}" \
  --argjson samples "${samples_json}" \
  '{
    version: $version,
    generatedAt: $generatedAt,
    tool: "sec4",
    identity: {
      policyProfileSha256: $policyProfileSha256,
      policyIdentityHash: $policyIdentityHash,
      compilerIdentityHash: $compilerIdentityHash,
      runtimeIdentityHash: $runtimeIdentityHash
    },
    artifacts: {
      checksums: $checksums,
      summary: $summary,
      policyFile: $policyFile,
      runtimeHeader: $runtimeHeader,
      runtimeSource: $runtimeSource,
      samples: $samples
    }
  }' > "${OUT_PATH}"

echo "wrote ${OUT_PATH}"
