#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/release-contract-smoke.yml"

if [[ ! -f "${workflow_path}" ]]; then
  echo "missing workflow file: ${workflow_path}" >&2
  exit 1
fi

require_token() {
  local token="$1"
  if ! grep -q -- "${token}" "${workflow_path}"; then
    echo "missing workflow contract token '${token}' in ${workflow_path}" >&2
    exit 1
  fi
}

require_token 'scripts/test-alpha-release-workflow-contract.sh'
require_token 'scripts/test-verify-release-promotion-inputs.sh'
require_token 'scripts/test-generate-release-publish-manifest.sh'
require_token 'scripts/test-verify-release-publish-manifest.sh'

echo "release-contract-smoke workflow contract test passed"
