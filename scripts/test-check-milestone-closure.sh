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

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "targetRps": 1, "requestsPerSec": 1, "p99": "1ms" },
        { "impl": "go", "targetRps": 1, "requestsPerSec": 1, "p99": "1ms" },
        { "impl": "node", "targetRps": 1, "requestsPerSec": 1, "p99": "1ms" },
        { "impl": "rust", "targetRps": 1, "requestsPerSec": 1, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "targetRps": 1, "requestsPerSec": 1, "p99": "1ms" }
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

echo "check-milestone-closure test passed"
