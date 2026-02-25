#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_in="$(mktemp)"
tmp_out="$(mktemp)"
trap 'rm -f "$tmp_in" "$tmp_out"' EXIT

cat >"$tmp_in" <<'JSON'
{
  "mode": "alpha-postgres-comparison-suite-repeats",
  "generatedAt": "2026-02-26T00:00:00Z",
  "dryRun": false,
  "runCount": 2,
  "runs": [
    {"run":"run-001","dryRun":false,"summary":"a"},
    {"run":"run-002","dryRun":false,"summary":"b"}
  ],
  "baselineStats": [
    {
      "impl": "sec4",
      "endpoint": "ping",
      "samples": 2,
      "requestsPerSec": {"mean": 10100.12, "min": 10000.01, "max": 10200.22},
      "p99Ms": {"mean": 4.5, "min": 4.2, "max": 4.9},
      "rssKb": {"mean": 42000, "min": 41000, "max": 43000}
    }
  ],
  "dbHotStats": [
    {
      "impl": "sec4-lasm",
      "endpoint": "db-hot-write",
      "samples": 2,
      "requestsPerSec": {"mean": 5100.5, "min": 5000.4, "max": 5200.6},
      "p99Ms": {"mean": 6.7, "min": 6.1, "max": 7.2},
      "rssKb": {"mean": 50000, "min": 49500, "max": 50500}
    }
  ]
}
JSON

out="$($root_dir/scripts/render_alpha_postgres_comparison_suite_repeats_summary.sh "$tmp_in" "$tmp_out")"
if ! grep -q "^wrote ${tmp_out}$" <<<"$out"; then
  echo "missing wrote marker" >&2
  exit 1
fi
if [ ! -f "$tmp_out" ]; then
  echo "expected rendered markdown output file to exist" >&2
  exit 1
fi

if ! grep -q '^# Alpha Postgres Repeated-Run Summary (v0.1)$' "$tmp_out"; then
  echo "missing markdown heading" >&2
  exit 1
fi
if ! grep -q '| sec4 | ping | 2 | 10100.12 | 10000.01 | 10200.22 | 4.5 | 4.2 | 4.9 | 42000 | 41000 | 43000 |' "$tmp_out"; then
  echo "missing baseline table row" >&2
  exit 1
fi
if ! grep -q '| sec4-lasm | db-hot-write | 2 | 5100.5 | 5000.4 | 5200.6 | 6.7 | 6.1 | 7.2 | 50000 | 49500 | 50500 |' "$tmp_out"; then
  echo "missing db-hot table row" >&2
  exit 1
fi

echo "render_alpha_postgres_comparison_suite_repeats_summary test passed"
