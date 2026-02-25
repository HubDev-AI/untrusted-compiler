#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "usage: $0 <alpha-postgres-comparison-suite-repeats.json> [out.md]" >&2
  exit 2
fi

in_path="$1"
out_path="${2:-$(cd "$(dirname "$0")/.." && pwd)/results/alpha-postgres-comparison-suite-repeats.md}"

if [ ! -f "$in_path" ]; then
  echo "repeats summary not found: $in_path" >&2
  exit 2
fi

if ! jq -e '.mode == "alpha-postgres-comparison-suite-repeats" and (.runs | type == "array")' "$in_path" >/dev/null; then
  echo "invalid repeats summary contract: $in_path" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"

generated_at="$(jq -r '.generatedAt // "unknown"' "$in_path")"
run_count="$(jq -r '.runCount // 0' "$in_path")"
dry_run="$(jq -r '.dryRun // false' "$in_path")"

render_stats_table() {
  local json_path="$1"
  local field="$2"
  jq -r --arg field "$field" '
    def fmt:
      if . == null then "n/a"
      elif (type == "number") then (((. * 100) | round) / 100 | tostring)
      else tostring
      end;
    (.[ $field ] // [])
    | .[]
    | [
        .impl,
        .endpoint,
        (.samples | tostring),
        (.requestsPerSec.mean | fmt),
        (.requestsPerSec.min | fmt),
        (.requestsPerSec.max | fmt),
        (.p99Ms.mean | fmt),
        (.p99Ms.min | fmt),
        (.p99Ms.max | fmt),
        (.rssKb.mean | fmt),
        (.rssKb.min | fmt),
        (.rssKb.max | fmt)
      ]
    | "| " + join(" | ") + " |"
  ' "$json_path"
}

{
  echo "# Alpha Postgres Repeated-Run Summary (v0.1)"
  echo
  echo "- Source: ${in_path}"
  echo "- Generated at: ${generated_at}"
  echo "- Run count: ${run_count}"
  echo "- Dry run: ${dry_run}"
  echo

  if [ "$dry_run" = "true" ]; then
    echo "_Dry-run mode: benchmark execution stats are not available._"
    echo
  fi

  echo "## Baseline Stats"
  echo
  echo "| Impl | Endpoint | Samples | RPS Mean | RPS Min | RPS Max | p99 Mean (ms) | p99 Min (ms) | p99 Max (ms) | RSS Mean (KB) | RSS Min (KB) | RSS Max (KB) |"
  echo "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
  render_stats_table "$in_path" "baselineStats"
  echo

  echo "## DB Hot Stats"
  echo
  echo "| Impl | Endpoint | Samples | RPS Mean | RPS Min | RPS Max | p99 Mean (ms) | p99 Min (ms) | p99 Max (ms) | RSS Mean (KB) | RSS Min (KB) | RSS Max (KB) |"
  echo "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
  render_stats_table "$in_path" "dbHotStats"
  echo
} >"$out_path"

echo "wrote ${out_path}"
