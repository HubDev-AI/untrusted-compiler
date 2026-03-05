#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "usage: $0 <workbench-full-benchmark-repeats.json> [out.md]" >&2
  exit 2
fi

in_path="$1"
out_path="${2:-$(cd "$(dirname "$0")/.." && pwd)/results/workbench-full-benchmark-repeats.md}"

if [ ! -f "$in_path" ]; then
  echo "repeats summary not found: $in_path" >&2
  exit 2
fi

if ! jq -e '.mode == "workbench-full-benchmark-suite-repeats" and (.runs | type == "array")' "$in_path" >/dev/null; then
  echo "invalid repeats summary contract: $in_path" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"

generated_at="$(jq -r '.generatedAt // "unknown"' "$in_path")"
run_count="$(jq -r '.runCount // 0' "$in_path")"
dry_run="$(jq -r '.dryRun // false' "$in_path")"

render_compare_stats_table() {
  jq -r '
    def fmt:
      if . == null then "n/a"
      elif (type == "number") then (((. * 100) | round) / 100 | tostring)
      else tostring
      end;
    (.compareStats // [])[]
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
  ' "$in_path"
}

render_step_stats_table() {
  jq -r '
    def fmt:
      if . == null then "n/a"
      elif (type == "number") then (((. * 100) | round) / 100 | tostring)
      else tostring
      end;
    (.stepStats // [])[]
    | [
        .impl,
        .endpoint,
        (.samples | tostring),
        (.kneeDetectedCount | tostring),
        (((.kneeDetectedRatio // 0) * 100) | fmt),
        (.kneeAtTargetRps.mean | fmt),
        (.kneeAtTargetRps.min | fmt),
        (.kneeAtTargetRps.max | fmt),
        (.achievedRatioMin.mean | fmt),
        (.achievedRatioMax.mean | fmt),
        (.p99MinMs.mean | fmt),
        (.p99MaxMs.mean | fmt)
      ]
    | "| " + join(" | ") + " |"
  ' "$in_path"
}

{
  echo "# Workbench Full-Suite Repeated-Run Summary (v0.1)"
  echo
  echo "- Source: ${in_path}"
  echo "- Generated at: ${generated_at}"
  echo "- Run count: ${run_count}"
  echo "- Dry run: ${dry_run}"
  echo

  echo "## Runs"
  echo
  echo "| Run | Summary | Compare | Step Matrix | Report |"
  echo "|---|---|---|---|---|"
  jq -r '.runs[] | "| " + .run + " | `" + .summary + "` | `" + .compareMatrix + "` | `" + .stepMatrix + "` | `" + .report + "` |"' "$in_path"
  echo

  if [ "$dry_run" = "true" ]; then
    echo "_Dry-run mode: aggregate performance stats are not available._"
    echo
  fi

  echo "## Compare Stats (Across Runs)"
  echo
  echo "| Impl | Endpoint | Samples | RPS Mean | RPS Min | RPS Max | p99 Mean (ms) | p99 Min (ms) | p99 Max (ms) | RSS Mean (KB) | RSS Min (KB) | RSS Max (KB) |"
  echo "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
  render_compare_stats_table
  echo

  echo "## Step Stats (Across Runs)"
  echo
  echo "| Impl | Endpoint | Samples | Knee Count | Knee Ratio (%) | Knee Target Mean | Knee Target Min | Knee Target Max | Achieved Min Mean | Achieved Max Mean | p99Min Mean (ms) | p99Max Mean (ms) |"
  echo "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
  render_step_stats_table
  echo
} >"$out_path"

echo "wrote ${out_path}"
