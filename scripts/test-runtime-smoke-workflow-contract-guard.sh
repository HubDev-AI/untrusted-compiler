#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-runtime-smoke-workflow-contract.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

workflow_path="${tmp_dir}/runtime-smoke.yml"

cat > "${workflow_path}" <<'YAML'
name: Runtime Smoke

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  runtime-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Run sec4 hello-api operator smoke
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke
      - name: Validate runtime smoke artifacts
        run: scripts/check-runtime-smoke-artifacts.sh --artifacts-dir build/runtime-smoke
      - name: Upload runtime smoke artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: runtime-smoke-artifacts
          path: build/runtime-smoke
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Runtime Smoke

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  runtime-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run sec4 hello-api operator smoke
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke
YAML

if "${contract_script}" --workflow "${workflow_path}" >"${tmp_dir}/guard.log" 2>&1; then
  echo "expected runtime-smoke workflow contract to fail when artifact validation step is missing" >&2
  exit 1
fi

if ! rg -Fq 'missing runtime-smoke workflow token: scripts/check-runtime-smoke-artifacts.sh --artifacts-dir build/runtime-smoke' "${tmp_dir}/guard.log"; then
  echo "expected missing-token diagnostic for runtime smoke artifact validation step" >&2
  exit 1
fi

echo "runtime-smoke workflow contract guard test passed"
