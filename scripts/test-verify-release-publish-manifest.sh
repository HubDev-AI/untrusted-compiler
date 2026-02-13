#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sec4-publish-manifest-verify-test.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARTIFACTS_DIR="${TMP_DIR}/artifacts"
MANIFEST_PATH="${ARTIFACTS_DIR}/publish-manifest.json"

"${ROOT_DIR}/scripts/release-alpha-gate.sh" --skip-tests --out-dir "${ARTIFACTS_DIR}" >/dev/null
"${ROOT_DIR}/scripts/verify-release-promotion-inputs.sh" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null
"${ROOT_DIR}/scripts/generate-release-publish-manifest.sh" --artifacts-dir "${ARTIFACTS_DIR}" --out "${MANIFEST_PATH}" >/dev/null
"${ROOT_DIR}/scripts/verify-release-publish-manifest.sh" --manifest "${MANIFEST_PATH}" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null

jq '.identity.runtimeIdentityHash = "rt_tampered"' "${MANIFEST_PATH}" > "${MANIFEST_PATH}.tmp"
mv "${MANIFEST_PATH}.tmp" "${MANIFEST_PATH}"

if "${ROOT_DIR}/scripts/verify-release-publish-manifest.sh" --manifest "${MANIFEST_PATH}" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null 2>&1; then
  echo "expected publish manifest verification to fail for tampered identity hash" >&2
  exit 1
fi

echo "verify-release-publish-manifest test passed"
