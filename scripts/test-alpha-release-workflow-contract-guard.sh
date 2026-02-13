#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-alpha-release-workflow-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

workflow_path="${tmp}/alpha-release-gate.yml"

cat > "${workflow_path}" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Verify release promotion inputs
        run: scripts/verify-release-promotion-inputs.sh build/release-alpha-gate/release-summary.txt
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
      - name: Upload alpha release gate artifacts
        uses: actions/upload-artifact@v4
        with:
          name: alpha-release-gate-artifacts
          path: build/release-alpha-gate
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
      - name: Upload alpha release gate artifacts
        uses: actions/upload-artifact@v4
        with:
          name: alpha-release-gate-artifacts
          path: build/release-alpha-gate
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when alpha workflow misses promotion verifier step" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Verify release promotion inputs
        run: scripts/verify-release-promotion-inputs.sh build/release-alpha-gate/release-summary.txt
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when alpha workflow misses artifact upload contract" >&2
  exit 1
fi

echo "alpha-release workflow guard test passed"
