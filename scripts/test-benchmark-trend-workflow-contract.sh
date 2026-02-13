#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/benchmark-trend.yml"

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

require_token 'schedule:'
require_token 'cron:'
require_token 'scripts/check-benchmark-evidence-quality.sh'
require_token '--fail-on-warning'
require_token 'benchmark-suite/scripts/check_regression_thresholds.sh'
require_token 'uses: actions/upload-artifact@v4'
require_token 'name: benchmark-trend-'
require_token 'path: benchmark-suite/results'

echo "benchmark trend workflow contract test passed"
