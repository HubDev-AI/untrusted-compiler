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

      - name: Run sec4 hello-api operator smoke (default)
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default
      - name: Run sec4 hello-api operator smoke (max-body)
        run: scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body
      - name: Run sec4 LASM DB adapter operator smoke (records-log)
        run: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log --artifacts-dir build/runtime-smoke/lasm-db-records-log
      - name: Run sec4 LASM DB adapter operator smoke (sqlite)
        run: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite --artifacts-dir build/runtime-smoke/lasm-db-sqlite
      - name: Validate runtime smoke bundle
        run: scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json
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
      - name: Run sec4 hello-api operator smoke (default)
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default
      - name: Run sec4 hello-api operator smoke (max-body)
        run: scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body
      - name: Run sec4 LASM DB adapter operator smoke (records-log)
        run: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log --artifacts-dir build/runtime-smoke/lasm-db-records-log
YAML

if "${contract_script}" --workflow "${workflow_path}" >"${tmp_dir}/guard.log" 2>&1; then
  echo "expected runtime-smoke workflow contract to fail when sqlite db-adapter smoke step is missing" >&2
  exit 1
fi

if ! rg -Fq 'missing runtime-smoke workflow token: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite --artifacts-dir build/runtime-smoke/lasm-db-sqlite' "${tmp_dir}/guard.log"; then
  echo "expected missing-token diagnostic for sqlite db-adapter smoke step" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Runtime Smoke

on:
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
      - name: Run sec4 hello-api operator smoke (default)
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default
      - name: Run sec4 hello-api operator smoke (max-body)
        run: scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body
      - name: Run sec4 LASM DB adapter operator smoke (records-log)
        run: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log --artifacts-dir build/runtime-smoke/lasm-db-records-log
      - name: Run sec4 LASM DB adapter operator smoke (sqlite)
        run: scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite --artifacts-dir build/runtime-smoke/lasm-db-sqlite
YAML

if "${contract_script}" --workflow "${workflow_path}" >"${tmp_dir}/bundle-guard.log" 2>&1; then
  echo "expected runtime-smoke workflow contract to fail when bundle-check step is missing" >&2
  exit 1
fi

if ! rg -Fq 'missing runtime-smoke workflow token: scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json' "${tmp_dir}/bundle-guard.log"; then
  echo "expected missing-token diagnostic for runtime smoke bundle-check step" >&2
  exit 1
fi

echo "runtime-smoke workflow contract guard test passed"
