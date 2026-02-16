#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 <step_summary.json> <out_analysis.json> [knee_ratio_threshold]" >&2
  exit 2
fi

step_path="$1"
out_path="$2"
knee_threshold="${3:-0.9}"

if [ ! -f "$step_path" ]; then
  echo "step summary file not found: ${step_path}" >&2
  exit 2
fi

if ! jq -e '.steps and (.steps | length > 0)' "$step_path" >/dev/null; then
  echo "invalid or empty step summary: ${step_path}" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"

jq -n \
  --argjson threshold "$knee_threshold" \
  --argjson input "$(cat "$step_path")" \
  '
  ($input.steps | map({
    targetRps: (.targetRps // 0),
    requestsPerSec: (.requestsPerSec // 0),
    p99: (.latency.p99 // ""),
    achievedRatio: (
      if (.targetRps // 0) > 0 then
        ((.requestsPerSec // 0) / (.targetRps // 0))
      else
        0
      end
    )
  })) as $rows
  | ($rows | map(.achievedRatio)) as $ratios
  | ($rows | map(.p99 | tostring | (try capture("(?<n>[0-9]+(\\.[0-9]+)?)").n catch null) // "0" | tonumber)) as $p99vals
  | ($rows | map(select(.achievedRatio < $threshold)) | .[0] // null) as $knee
  | {
      version: "0.1",
      impl: ($input.impl // "unknown"),
      endpoint: ($input.endpoint // "unknown"),
      kneeThreshold: $threshold,
      stepCount: ($rows | length),
      steps: $rows,
      summary: {
        achievedRatioMin: ($ratios | min),
        achievedRatioMax: ($ratios | max),
        p99MinMs: ($p99vals | min),
        p99MaxMs: ($p99vals | max),
        kneeDetected: ($knee != null),
        kneeAtTargetRps: ($knee.targetRps // null),
        kneeObservedRps: ($knee.requestsPerSec // null)
      }
    }
  ' > "$out_path"

echo "wrote $out_path"
