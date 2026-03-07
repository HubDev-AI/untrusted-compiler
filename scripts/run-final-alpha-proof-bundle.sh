#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_DIR="${ROOT_DIR}/build/release-alpha-gate"
PROFILE_PATH="${ROOT_DIR}/policies/default-secure-prod.sec4.policy"
SKIP_TESTS=0
DRY_RUN=0

usage() {
  cat <<'USAGE'
usage: scripts/run-final-alpha-proof-bundle.sh [options]

Runs the final alpha proof-bundle chain on one artifact directory:
  1) release-alpha-gate
  2) verify-release-promotion-inputs
  3) generate-release-publish-manifest
  4) verify-release-publish-manifest

Options:
  --artifacts-dir <path>  Output directory for release artifacts (default: build/release-alpha-gate)
  --profile <path>        Policy profile for release-alpha-gate (default: policies/default-secure-prod.sec4.policy)
  --skip-tests            Pass --skip-tests to release-alpha-gate
  --dry-run               Print exact command sequence without executing
  -h, --help              Show this help
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
    --profile)
      PROFILE_PATH="$2"
      shift 2
      ;;
    --profile=*)
      PROFILE_PATH="${1#--profile=}"
      shift
      ;;
    --skip-tests)
      SKIP_TESTS=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
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

print_cmd() {
  printf 'run:'
  printf ' %q' "$@"
  printf '\n'
}

run_cmd() {
  print_cmd "$@"
  "$@"
}

release_cmd=(
  "${ROOT_DIR}/scripts/release-alpha-gate.sh"
  --profile "${PROFILE_PATH}"
  --out-dir "${ARTIFACTS_DIR}"
)
if [[ "${SKIP_TESTS}" -eq 1 ]]; then
  release_cmd+=(--skip-tests)
fi

verify_inputs_cmd=(
  "${ROOT_DIR}/scripts/verify-release-promotion-inputs.sh"
  --artifacts-dir "${ARTIFACTS_DIR}"
)

generate_manifest_cmd=(
  "${ROOT_DIR}/scripts/generate-release-publish-manifest.sh"
  --artifacts-dir "${ARTIFACTS_DIR}"
  --out "${ARTIFACTS_DIR}/publish-manifest.json"
)

verify_manifest_cmd=(
  "${ROOT_DIR}/scripts/verify-release-publish-manifest.sh"
  --manifest "${ARTIFACTS_DIR}/publish-manifest.json"
  --artifacts-dir "${ARTIFACTS_DIR}"
)

if [[ "${DRY_RUN}" -eq 1 ]]; then
  print_cmd "${release_cmd[@]}"
  print_cmd "${verify_inputs_cmd[@]}"
  print_cmd "${generate_manifest_cmd[@]}"
  print_cmd "${verify_manifest_cmd[@]}"
  exit 0
fi

run_cmd "${release_cmd[@]}"
run_cmd "${verify_inputs_cmd[@]}"
run_cmd "${generate_manifest_cmd[@]}"
run_cmd "${verify_manifest_cmd[@]}"

echo "ok: final alpha proof bundle completed"
echo "artifacts: ${ARTIFACTS_DIR}"
echo "summary: ${ARTIFACTS_DIR}/summary.txt"
echo "checksums: ${ARTIFACTS_DIR}/checksums.txt"
echo "manifest: ${ARTIFACTS_DIR}/publish-manifest.json"
