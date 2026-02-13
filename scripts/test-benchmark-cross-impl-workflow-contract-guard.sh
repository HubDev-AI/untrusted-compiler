#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-benchmark-cross-impl-workflow-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

workflow_path="${tmp}/benchmark-cross-impl-evidence.yml"

cat > "${workflow_path}" <<'YAML'
name: Benchmark Cross-Impl Evidence
on:
  workflow_dispatch:
jobs:
  cross-impl-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run cross-impl benchmark matrix (ping+decode)
        run: |
          benchmark-suite/scripts/run_comparison_matrix.sh \
            --impls sec4,node,go,rust \
            --endpoints ping,decode
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Upload cross-impl benchmark evidence
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-cross-impl-evidence
          path: benchmark-suite/results
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Benchmark Cross-Impl Evidence
on:
  workflow_dispatch:
jobs:
  cross-impl-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run cross-impl benchmark matrix (ping+decode)
        run: |
          benchmark-suite/scripts/run_comparison_matrix.sh \
            --impls sec4,node,go,rust \
            --endpoints ping,decode
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json
      - name: Upload cross-impl benchmark evidence
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-cross-impl-evidence
          path: benchmark-suite/results
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when cross-impl workflow misses strict quality flag" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Benchmark Cross-Impl Evidence
on:
  workflow_dispatch:
jobs:
  cross-impl-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run cross-impl benchmark matrix (ping+decode)
        run: |
          benchmark-suite/scripts/run_comparison_matrix.sh \
            --impls sec4,node,go,rust \
            --endpoints ping,decode
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when cross-impl workflow misses artifact upload contract" >&2
  exit 1
fi

echo "benchmark cross-impl workflow guard test passed"
