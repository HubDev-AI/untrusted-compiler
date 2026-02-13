#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/benchmark-smoke.yml"

if ! grep -q 'scripts/check-milestone-closure.sh' "${workflow_path}"; then
  echo "missing closure-audit command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q 'scripts/test-check-milestone-closure.sh' "${workflow_path}"; then
  echo "missing closure fixture test command in ${workflow_path}" >&2
  exit 1
fi

if ! grep -q -- '--fail-on-pending' "${workflow_path}"; then
  echo "missing strict closure flag (--fail-on-pending) in ${workflow_path}" >&2
  exit 1
fi

echo "benchmark-smoke closure-gate test passed"
