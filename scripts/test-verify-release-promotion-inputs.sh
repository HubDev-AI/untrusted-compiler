#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sec4-release-verify-test.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARTIFACTS_DIR="${TMP_DIR}/artifacts"

"${ROOT_DIR}/scripts/release-alpha-gate.sh" --skip-tests --out-dir "${ARTIFACTS_DIR}" >/dev/null
"${ROOT_DIR}/scripts/verify-release-promotion-inputs.sh" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null

jq '.build.compilerHash = "cpl_tampered"' "${ARTIFACTS_DIR}/hello-audit.json" > "${ARTIFACTS_DIR}/hello-audit.json.tmp"
mv "${ARTIFACTS_DIR}/hello-audit.json.tmp" "${ARTIFACTS_DIR}/hello-audit.json"

if "${ROOT_DIR}/scripts/verify-release-promotion-inputs.sh" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null 2>&1; then
  echo "expected release promotion verifier to fail for tampered audit artifact" >&2
  exit 1
fi

echo "release promotion input verifier test passed"
