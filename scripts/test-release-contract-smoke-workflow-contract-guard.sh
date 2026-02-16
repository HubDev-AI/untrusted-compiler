#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-release-contract-smoke-workflow-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

workflow_path="${tmp}/release-contract-smoke.yml"

cat > "${workflow_path}" <<'YAML'
name: Release Contract Smoke
on:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Release Contract Smoke
on:
  workflow_dispatch:
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when workflow trigger coverage is incomplete" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Release Contract Smoke
on:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when workflow misses publish verifier step" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Release Contract Smoke
on:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when workflow misses release-contract guard step" >&2
  exit 1
fi

echo "release-contract-smoke workflow guard test passed"
