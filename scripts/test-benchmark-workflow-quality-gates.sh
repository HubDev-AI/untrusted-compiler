#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

assert_quality_gate() {
  local workflow_path="$1"

  if ! grep -q '^\s*- name: Enforce benchmark evidence quality$' "${workflow_path}"; then
    echo "missing quality-gate step in ${workflow_path}" >&2
    exit 1
  fi

  if ! grep -q 'scripts/check-benchmark-evidence-quality.sh' "${workflow_path}"; then
    echo "missing benchmark evidence quality command in ${workflow_path}" >&2
    exit 1
  fi

  if ! grep -q -- '--fail-on-warning' "${workflow_path}"; then
    echo "missing strict quality flag (--fail-on-warning) in ${workflow_path}" >&2
    exit 1
  fi
}

assert_quality_gate "${root_dir}/.github/workflows/benchmark-trend.yml"
assert_quality_gate "${root_dir}/.github/workflows/benchmark-cross-impl-evidence.yml"

echo "benchmark workflow quality-gate test passed"
