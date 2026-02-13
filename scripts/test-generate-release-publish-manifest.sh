#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sec4-publish-manifest-test.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARTIFACTS_DIR="${TMP_DIR}/artifacts"
MANIFEST_PATH="${ARTIFACTS_DIR}/publish-manifest.json"

"${ROOT_DIR}/scripts/release-alpha-gate.sh" --skip-tests --out-dir "${ARTIFACTS_DIR}" >/dev/null
"${ROOT_DIR}/scripts/verify-release-promotion-inputs.sh" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null
"${ROOT_DIR}/scripts/generate-release-publish-manifest.sh" --artifacts-dir "${ARTIFACTS_DIR}" --out "${MANIFEST_PATH}" >/dev/null

if [ ! -f "${MANIFEST_PATH}" ]; then
  echo "expected publish manifest to be generated" >&2
  exit 1
fi

if ! jq -e '
  .version == "0.1"
  and .tool == "sec4"
  and (.identity | has("policyIdentityHash") and has("compilerIdentityHash") and has("runtimeIdentityHash"))
  and (.artifacts.samples | length > 0)
' "${MANIFEST_PATH}" >/dev/null; then
  echo "publish manifest missing required fields" >&2
  exit 1
fi

grep -v '^policy_identity_hash ' "${ARTIFACTS_DIR}/checksums.txt" > "${ARTIFACTS_DIR}/checksums.txt.tmp"
mv "${ARTIFACTS_DIR}/checksums.txt.tmp" "${ARTIFACTS_DIR}/checksums.txt"

if "${ROOT_DIR}/scripts/generate-release-publish-manifest.sh" --artifacts-dir "${ARTIFACTS_DIR}" --out "${MANIFEST_PATH}" >/dev/null 2>&1; then
  echo "expected manifest generation to fail when required checksum entry is missing" >&2
  exit 1
fi

echo "generate-release-publish-manifest test passed"
