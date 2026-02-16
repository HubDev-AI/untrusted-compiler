#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--workflow <path>]

Validates operator-handoff workflow contract.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/operator-handoff-smoke.yml"

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
  echo "missing operator-handoff workflow: ${workflow_path}" >&2
  exit 1
fi

require_token() {
  local token="$1"
  if ! rg -Fq -- "${token}" "${workflow_path}"; then
    echo "missing operator-handoff workflow token: ${token}" >&2
    exit 1
  fi
}

require_token 'push:'
require_token 'branches:'
require_token '- main'
require_token 'runs-on: ubuntu-latest'
require_token 'uses: actions/checkout@v4'
require_token 'scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke'
require_token 'if: always()'
require_token 'uses: actions/upload-artifact@v4'
require_token 'name: operator-handoff-smoke-artifacts'
require_token 'path: build/operator-handoff-smoke'

echo "operator-handoff workflow contract test passed"
