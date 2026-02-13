#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE_PATH="${ROOT_DIR}/policies/default-secure-prod.sec4.policy"
OUT_DIR="${ROOT_DIR}/build/release-alpha-gate"
RUN_TESTS=1

usage() {
  cat <<'USAGE'
Usage: scripts/release-alpha-gate.sh [options]

Options:
  --profile <path>      Policy profile to apply as sec4.policy in sample workspaces
  --out-dir <path>      Output directory for captured release artifacts
  --skip-tests          Skip cargo test phase (not recommended for release candidates)
  -h, --help            Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE_PATH="$2"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="$2"
      shift 2
      ;;
    --skip-tests)
      RUN_TESTS=0
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

if [[ ! -f "${PROFILE_PATH}" ]]; then
  echo "error: policy profile not found: ${PROFILE_PATH}" >&2
  exit 2
fi

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

run() {
  echo "+ $*"
  "$@"
}

SEC4_BIN="${ROOT_DIR}/target/debug/sec4"
if [[ ! -x "${SEC4_BIN}" ]]; then
  run cargo build -p sec4 --manifest-path "${ROOT_DIR}/Cargo.toml" >/dev/null
fi

run "${ROOT_DIR}/scripts/check-naming-lock.sh"

if [[ "${RUN_TESTS}" -eq 1 ]]; then
  run cargo test -q --manifest-path "${ROOT_DIR}/Cargo.toml"
fi

run cargo build -p sec4 --manifest-path "${ROOT_DIR}/Cargo.toml" >/dev/null

mkdir -p "${OUT_DIR}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sec4-alpha-gate.XXXXXX")"
trap 'rm -rf "${WORK_DIR}"' EXIT

CHECKSUMS_FILE="${OUT_DIR}/checksums.txt"
: > "${CHECKSUMS_FILE}"

SAMPLES=("hello" "hello-api")
for sample in "${SAMPLES[@]}"; do
  SOURCE_DIR="${ROOT_DIR}/examples/${sample}"
  SAMPLE_DIR="${WORK_DIR}/${sample}"

  if [[ ! -d "${SOURCE_DIR}" ]]; then
    echo "error: missing sample project directory: ${SOURCE_DIR}" >&2
    exit 2
  fi

  run cp -R "${SOURCE_DIR}" "${SAMPLE_DIR}"
  run cp "${PROFILE_PATH}" "${SAMPLE_DIR}/sec4.policy"

  run "${SEC4_BIN}" build --path "${SAMPLE_DIR}" --locked
  run "${SEC4_BIN}" build --path "${SAMPLE_DIR}" --sbom

  META_PATH="${SAMPLE_DIR}/build/build_metadata.json"
  SBOM_PATH="${SAMPLE_DIR}/build/sbom.json"

  if [[ ! -f "${META_PATH}" || ! -f "${SBOM_PATH}" ]]; then
    echo "error: required build artifacts were not produced for sample '${sample}'" >&2
    exit 1
  fi

  META_HASH_BEFORE="$(hash_file "${META_PATH}")"
  SBOM_HASH_BEFORE="$(hash_file "${SBOM_PATH}")"

  run "${SEC4_BIN}" build --path "${SAMPLE_DIR}" --sbom

  META_HASH_AFTER="$(hash_file "${META_PATH}")"
  SBOM_HASH_AFTER="$(hash_file "${SBOM_PATH}")"

  if [[ "${META_HASH_BEFORE}" != "${META_HASH_AFTER}" ]]; then
    echo "error: build metadata is non-deterministic for sample '${sample}'" >&2
    exit 1
  fi

  if [[ "${SBOM_HASH_BEFORE}" != "${SBOM_HASH_AFTER}" ]]; then
    echo "error: sbom is non-deterministic for sample '${sample}'" >&2
    exit 1
  fi

  AUDIT_REPORT_PATH="${OUT_DIR}/${sample}-audit.json"
  run "${SEC4_BIN}" audit --path "${SAMPLE_DIR}" --format json --fail-on "risk>=HIGH" --write-report "${AUDIT_REPORT_PATH}" >/dev/null
  MAP_PATH="${SAMPLE_DIR}/build/security_map.json"
  if [[ ! -f "${MAP_PATH}" ]]; then
    echo "error: security map was not produced by audit for sample '${sample}'" >&2
    exit 1
  fi

  run cp "${META_PATH}" "${OUT_DIR}/${sample}-build_metadata.json"
  run cp "${SBOM_PATH}" "${OUT_DIR}/${sample}-sbom.json"
  run cp "${MAP_PATH}" "${OUT_DIR}/${sample}-security_map.json"

  {
    echo "${sample} build_metadata_sha256 ${META_HASH_BEFORE}"
    echo "${sample} sbom_sha256 ${SBOM_HASH_BEFORE}"
  } >> "${CHECKSUMS_FILE}"
done

SUMMARY_PATH="${OUT_DIR}/summary.txt"
{
  echo "sec4 alpha release gate: PASS"
  echo "policy profile: ${PROFILE_PATH}"
  echo "samples: ${SAMPLES[*]}"
  echo "artifacts: ${OUT_DIR}"
  echo "checksums: ${CHECKSUMS_FILE}"
} > "${SUMMARY_PATH}"

echo "ok: alpha release gate passed"
echo "ok: artifacts captured in ${OUT_DIR}"
