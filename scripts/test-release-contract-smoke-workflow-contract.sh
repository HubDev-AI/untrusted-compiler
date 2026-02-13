#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/release-contract-smoke.yml"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --workflow)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --workflow" >&2
        exit 2
      fi
      workflow_path="$2"
      shift 2
      ;;
    --workflow=*)
      workflow_path="${1#--workflow=}"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

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

require_regex() {
  local pattern="$1"
  if ! rg -q -- "${pattern}" "${workflow_path}"; then
    echo "missing workflow contract pattern '${pattern}' in ${workflow_path}" >&2
    exit 1
  fi
}

require_regex '^[[:space:]]*pull_request:[[:space:]]*$'
require_regex '^[[:space:]]*push:[[:space:]]*$'
require_regex '^[[:space:]]*branches:[[:space:]]*$'
require_regex '^[[:space:]]*-[[:space:]]*main[[:space:]]*$'

require_token 'scripts/test-alpha-release-workflow-contract.sh'
require_token 'scripts/test-verify-release-promotion-inputs.sh'
require_token 'scripts/test-generate-release-publish-manifest.sh'
require_token 'scripts/test-verify-release-publish-manifest.sh'

echo "release-contract-smoke workflow contract test passed"
