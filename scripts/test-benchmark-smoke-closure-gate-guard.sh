#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-benchmark-smoke-closure-gate.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

workflow_path="${tmp}/benchmark-smoke.yml"

cat > "${workflow_path}" <<'YAML'
name: Benchmark Smoke
on:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/test-check-milestone-closure.sh
          scripts/check-milestone-closure.sh --fail-on-pending
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Benchmark Smoke
on:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/check-milestone-closure.sh --fail-on-pending
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when benchmark-smoke workflow misses closure fixture test command" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Benchmark Smoke
on:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/test-check-milestone-closure.sh
          scripts/check-milestone-closure.sh
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when benchmark-smoke workflow misses strict closure flag" >&2
  exit 1
fi

echo "benchmark-smoke closure-gate guard test passed"
