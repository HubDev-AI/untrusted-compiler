#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/benchmark-smoke.yml"

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

if ! grep -q 'scripts/check-milestone-closure.sh' "${workflow_path}"; then
  echo "missing closure-audit command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-check-milestone-closure.sh' "${workflow_path}"; then
  echo "missing closure fixture test command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-benchmark-smoke-closure-gate-guard.sh' "${workflow_path}"; then
  echo "missing closure guard test command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-benchmark-cross-impl-workflow-contract.sh' "${workflow_path}"; then
  echo "missing cross-impl workflow contract command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-benchmark-cross-impl-workflow-contract-guard.sh' "${workflow_path}"; then
  echo "missing cross-impl workflow guard command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-benchmark-trend-workflow-contract.sh' "${workflow_path}"; then
  echo "missing trend workflow contract command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-benchmark-trend-workflow-contract-guard.sh' "${workflow_path}"; then
  echo "missing trend workflow guard command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q -- '--fail-on-pending' "${workflow_path}"; then
  echo "missing strict closure flag (--fail-on-pending) in ${workflow_path}" >&2
  exit 1
fi

echo "benchmark-smoke closure-gate test passed"
