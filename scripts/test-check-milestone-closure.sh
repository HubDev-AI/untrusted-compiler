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
cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
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
cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
YAML
cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
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

audit_output="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending)"
if printf '%s\n' "$audit_output" | rg -q -- "$tmp"; then
  echo "expected repo-relative evidence paths in closure audit output" >&2
  exit 1
fi

audit_json="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json --fail-on-pending)"
if ! printf '%s\n' "$audit_json" | jq -e '.overall == "PASS" and .pendingCount == 0' >/dev/null; then
  echo "expected PASS json closure summary for passing fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M9-H") != null' >/dev/null; then
  echo "expected json closure output to include M9-H gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M10-D") != null' >/dev/null; then
  echo "expected json closure output to include M10-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M13-D") != null' >/dev/null; then
  echo "expected json closure output to include M13-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M13-E") != null' >/dev/null; then
  echo "expected json closure output to include M13-E gate" >&2
  exit 1
fi
if printf '%s\n' "$audit_json" | rg -q -- "$tmp"; then
  echo "expected repo-relative evidence paths in json closure output" >&2
  exit 1
fi

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note
MD

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without trend entry" >&2
  exit 1
fi

if pending_json="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json --fail-on-pending 2>/dev/null)"; then
  echo "expected pending failure exit code for json closure output without trend entry" >&2
  exit 1
fi
if ! printf '%s\n' "$pending_json" | jq -e '.overall == "PENDING" and .pendingCount > 0' >/dev/null; then
  echo "expected pending json closure summary for failing fixture" >&2
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

cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
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

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when alpha-release workflow misses artifact upload contract" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
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
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
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

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses alpha-release guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
YAML

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses cross-impl guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses benchmark-trend guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
YAML

cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
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

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when benchmark-smoke workflow misses closure fixture test coverage" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
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
