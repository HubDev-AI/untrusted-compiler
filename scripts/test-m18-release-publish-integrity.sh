#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
verify_script="${root_dir}/scripts/verify-release-publish-manifest.sh"
verify_test="${root_dir}/scripts/test-verify-release-publish-manifest.sh"

if [ ! -f "${verify_script}" ]; then
  echo "missing release publish manifest verifier script: ${verify_script}" >&2
  exit 1
fi
if [ ! -f "${verify_test}" ]; then
  echo "missing release publish manifest verifier test script: ${verify_test}" >&2
  exit 1
fi

require_token() {
  local path="$1"
  local token="$2"
  local label="$3"
  if ! rg -Fq -- "${token}" "${path}"; then
    echo "missing ${label} token in ${path}: ${token}" >&2
    exit 1
  fi
}

require_token "${verify_script}" 'hash_file()' 'artifact hash helper'
require_token "${verify_script}" 'assert_hash_matches()' 'artifact hash matcher helper'
require_token "${verify_script}" 'artifact checksum mismatch for ${label}' 'artifact mismatch diagnostic contract'
require_token "${verify_script}" '"policy profile copy"' 'policy hash validation'
require_token "${verify_script}" '"runtime header copy"' 'runtime header hash validation'
require_token "${verify_script}" '"runtime source copy"' 'runtime source hash validation'
require_token "${verify_script}" "sample '\${sample_name}' build metadata" 'sample build metadata hash validation'
require_token "${verify_script}" "sample '\${sample_name}' sbom" 'sample sbom hash validation'
require_token "${verify_script}" 'runtime_header_sha256' 'runtime header checksum contract'
require_token "${verify_script}" 'runtime_source_sha256' 'runtime source checksum contract'
require_token "${verify_test}" 'tampered sample sbom artifact' 'tampered sbom regression coverage'

echo "m18 release publish integrity contract test passed"
