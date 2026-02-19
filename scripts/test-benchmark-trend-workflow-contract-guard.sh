#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-benchmark-trend-workflow-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

workflow_path="${tmp}/benchmark-trend.yml"

cat > "${workflow_path}" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping \
            --max-rss-kb 500000
      - name: Upload benchmark trend artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-trend-node-ping
          path: benchmark-suite/results
YAML

"${contract_script}" --workflow "${workflow_path}" >/dev/null

cat > "${workflow_path}" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping \
            --max-rss-kb 500000
      - name: Upload benchmark trend artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-trend-node-ping
          path: benchmark-suite/results
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when benchmark-trend workflow misses strict quality flag" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping \
            --max-rss-kb 500000
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when benchmark-trend workflow misses artifact upload contract" >&2
  exit 1
fi

cat > "${workflow_path}" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
      - name: Upload benchmark trend artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-trend-node-ping
          path: benchmark-suite/results
YAML

if "${contract_script}" --workflow "${workflow_path}" >/dev/null 2>&1; then
  echo "expected contract failure when benchmark-trend workflow misses RSS threshold guard flag" >&2
  exit 1
fi

echo "benchmark trend workflow guard test passed"
