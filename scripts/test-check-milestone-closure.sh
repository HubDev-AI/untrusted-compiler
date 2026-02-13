#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/scripts" "$tmp/.github/workflows" "$tmp/benchmark-suite/results/summaries" "$tmp/docs/book"

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh --fail-on-pending
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"
touch "$tmp/scripts/verify-release-promotion-inputs.sh"
touch "$tmp/scripts/generate-release-publish-manifest.sh"
touch "$tmp/scripts/verify-release-publish-manifest.sh"
touch "$tmp/.github/workflows/alpha-release-gate.yml"
cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
YAML

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
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

cat > "$tmp/.github/workflows/benchmark-cross-impl-evidence.yml" <<'YAML'
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

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "rust", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note

## Trend Entry (2026-02-13)
MD

"$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note
MD

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without trend entry" >&2
  exit 1
fi

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note

## Trend Entry (2026-02-13)
MD

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without per-endpoint rust impl coverage" >&2
  exit 1
fi

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "rust", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without strict benchmark trend workflow quality gate" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
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
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without benchmark trend artifact upload step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
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

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when release gate omits strict closure flag" >&2
  exit 1
fi

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh --fail-on-pending
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"

cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when release-contract-smoke workflow misses publish verifier coverage" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
YAML

cat > "$tmp/.github/workflows/benchmark-cross-impl-evidence.yml" <<'YAML'
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

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when cross-impl workflow omits strict quality flag" >&2
  exit 1
fi

echo "check-milestone-closure test passed"
