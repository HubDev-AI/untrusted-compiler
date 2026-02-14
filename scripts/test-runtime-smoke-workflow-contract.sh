#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--workflow <path>]

Validates runtime-smoke workflow contract.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/runtime-smoke.yml"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --workflow)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      workflow_path="$2"
      shift 2
      ;;
    --workflow=*)
      workflow_path="${1#--workflow=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [ ! -f "${workflow_path}" ]; then
  echo "missing runtime-smoke workflow: ${workflow_path}" >&2
  exit 1
fi

require_token() {
  local token="$1"
  if ! rg -Fq -- "${token}" "${workflow_path}"; then
    echo "missing runtime-smoke workflow token: ${token}" >&2
    exit 1
  fi
}

require_token 'pull_request:'
require_token 'push:'
require_token 'branches:'
require_token '- main'
require_token 'runs-on: ubuntu-latest'
require_token 'uses: actions/checkout@v4'
require_token 'scripts/smoke-sec4-run-hello-api.sh'

echo "runtime-smoke workflow contract test passed"
