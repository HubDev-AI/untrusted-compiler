#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-operator-handoff-workflow-contract.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

workflow_path="${tmp_dir}/operator-handoff-smoke.yml"

cat > "${workflow_path}" <<'YAML'
name: Operator Handoff Smoke

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  operator-handoff-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 25
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run M17 operator handoff CI smoke wrapper
        run: scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke
      - name: Upload operator handoff smoke artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: operator-handoff-smoke-artifacts
          path: build/operator-handoff-smoke
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Operator Handoff Smoke

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  operator-handoff-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 25
    steps:
      - name: Checkout
        uses: actions/checkout@v4
YAML

if "${contract_script}" --workflow "${workflow_path}" >"${tmp_dir}/guard.log" 2>&1; then
  echo "expected operator-handoff workflow contract to fail when ci smoke step is missing" >&2
  exit 1
fi

if ! rg -Fq 'missing operator-handoff workflow token: scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke' "${tmp_dir}/guard.log"; then
  echo "expected missing-token diagnostic for operator-handoff ci smoke command" >&2
  exit 1
fi

echo "operator-handoff workflow contract guard test passed"
