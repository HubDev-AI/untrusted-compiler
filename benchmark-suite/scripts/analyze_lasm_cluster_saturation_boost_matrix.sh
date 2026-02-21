#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <saturation_boost_matrix.json> <out_analysis.json>" >&2
  exit 2
fi

matrix_path="$1"
out_path="$2"

if [ ! -f "${matrix_path}" ]; then
  echo "saturation boost matrix file not found: ${matrix_path}" >&2
  exit 2
fi

if ! jq -e '.runs | type == "array" and length > 0' "${matrix_path}" >/dev/null; then
  echo "matrix runs must be a non-empty array: ${matrix_path}" >&2
  exit 2
fi

if ! jq -e '
  .runs
  | all(
      (has("saturationBoostStep") and (.saturationBoostStep | type == "number") and .saturationBoostStep >= 1) and
      (has("pass") and (.pass | type == "boolean")) and
      (has("requests") and (.requests | type == "number") and .requests >= 0) and
      (has("requestsPerSec") and (.requestsPerSec | type == "number") and .requestsPerSec >= 0) and
      (has("peakRssKb") and (.peakRssKb | type == "number") and .peakRssKb >= 0) and
      ((has("p99") | not) or (.p99 | type == "string"))
    )
' "${matrix_path}" >/dev/null; then
  echo "matrix runs contain invalid fields: ${matrix_path}" >&2
  exit 2
fi

mkdir -p "$(dirname "${out_path}")"

jq -n \
  --arg sourceMatrix "${matrix_path}" \
  --slurpfile matrix "${matrix_path}" \
  '
  def p99_to_ms:
    if (.p99 | type) != "string" then 1000000000000
    elif (.p99 | test("^[0-9]+(\\.[0-9]+)?ms$")) then (.p99 | sub("ms$"; "") | tonumber)
    elif (.p99 | test("^[0-9]+(\\.[0-9]+)?us$")) then ((.p99 | sub("us$"; "") | tonumber) / 1000)
    elif (.p99 | test("^[0-9]+(\\.[0-9]+)?s$")) then ((.p99 | sub("s$"; "") | tonumber) * 1000)
    else 1000000000000
    end;
  ($matrix[0]) as $m
  | ($m.runs | map({
      saturationBoostStep: .saturationBoostStep,
      pass: .pass,
      requests: .requests,
      requestsPerSec: .requestsPerSec,
      peakRssKb: .peakRssKb,
      p99: (.p99 // ""),
      p99Ms: p99_to_ms,
      clusterRelayWorkersResolved: (.clusterRelayWorkersResolved // null),
      clusterAcceptWorkersResolved: (.clusterAcceptWorkersResolved // null),
      clusterRelayAcceptBatchMaxResolved: (.clusterRelayAcceptBatchMaxResolved // null),
      clusterRelayPumpBatchMaxResolved: (.clusterRelayPumpBatchMaxResolved // null),
      clusterRelayQueueCapacityResolved: (.clusterRelayQueueCapacityResolved // null),
      clusterRelayQueueShardCapacityResolved: (.clusterRelayQueueShardCapacityResolved // null),
      summaryFile: .summaryFile
    })) as $rows
  | ($rows
      | sort_by([
          (if .pass then 0 else 1 end),
          (-.requestsPerSec),
          (.p99Ms),
          (-.requests),
          (.peakRssKb),
          (.saturationBoostStep)
        ])) as $ranked
  | ($rows | map(select(.pass == true)) | length) as $pass_count
  | ($rows | length) as $run_count
  | {
      version: "0.1",
      sourceMatrix: $sourceMatrix,
      impl: ($m.impl // "sec4-lasm-cluster"),
      run: ($m.run // {}),
      boostSteps: ($m.boostSteps // []),
      summary: {
        runCount: $run_count,
        passCount: $pass_count,
        failCount: ($run_count - $pass_count),
        selectionMode: (if $pass_count > 0 then "pass-first" else "throughput-best-no-pass" end),
        recommendedBoostStep: ($ranked[0].saturationBoostStep)
      },
      recommended: $ranked[0],
      rankedRuns: $ranked
    }
  ' > "${out_path}"

echo "wrote ${out_path}"
