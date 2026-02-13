#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/scripts" "$tmp/.github/workflows" "$tmp/benchmark-suite/results/summaries" "$tmp/docs/book"

touch "$tmp/scripts/release-alpha-gate.sh"
touch "$tmp/scripts/verify-release-promotion-inputs.sh"
touch "$tmp/scripts/generate-release-publish-manifest.sh"
touch "$tmp/scripts/verify-release-publish-manifest.sh"
touch "$tmp/.github/workflows/alpha-release-gate.yml"

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

echo "check-milestone-closure test passed"
