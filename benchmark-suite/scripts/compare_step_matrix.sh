#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 4 ]; then
  echo "usage: $0 <summaries_dir> <out_step_matrix.json> [impls_csv] [endpoints_csv]" >&2
  exit 2
fi

summaries_dir="$1"
out_path="$2"
impls_csv="${3:-}"
endpoints_csv="${4:-}"

if [ ! -d "$summaries_dir" ]; then
  echo "summaries directory not found: ${summaries_dir}" >&2
  exit 2
fi

analysis_files=()

if [ -n "$impls_csv" ] && [ -n "$endpoints_csv" ]; then
  IFS=',' read -r -a impls <<< "$impls_csv"
  IFS=',' read -r -a endpoints <<< "$endpoints_csv"
  for raw_impl in "${impls[@]}"; do
    impl="${raw_impl// /}"
    [ -z "$impl" ] && continue
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="${raw_endpoint// /}"
      [ -z "$endpoint" ] && continue
      candidate="${summaries_dir}/${impl}-${endpoint}-step-analysis.json"
      if [ ! -f "$candidate" ]; then
        echo "missing step analysis for impl=${impl} endpoint=${endpoint}: ${candidate}" >&2
        exit 2
      fi
      analysis_files+=("$candidate")
    done
  done
else
  mapfile -t analysis_files < <(find "$summaries_dir" -maxdepth 1 -type f -name '*-step-analysis.json' | sort)
fi

if [ "${#analysis_files[@]}" -eq 0 ]; then
  echo "no step analysis files found in ${summaries_dir}" >&2
  exit 2
fi

rows_json="$(jq -s '
  [ .[]
    | {
        impl: (.impl // "unknown"),
        endpoint: (.endpoint // "unknown"),
        kneeDetected: (.summary.kneeDetected // false),
        kneeAtTargetRps: (.summary.kneeAtTargetRps // 0),
        kneeObservedRps: (.summary.kneeObservedRps // 0),
        achievedRatioMin: (.summary.achievedRatioMin // 0),
        achievedRatioMax: (.summary.achievedRatioMax // 0),
        p99MinMs: (.summary.p99MinMs // 0),
        p99MaxMs: (.summary.p99MaxMs // 0),
        stepCount: (.stepCount // 0)
      }
  ]
' "${analysis_files[@]}")"

if [ "$(jq 'length' <<<"$rows_json")" -eq 0 ]; then
  echo "no valid step analysis rows found in ${summaries_dir}" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"

jq -n \
  --argjson rows "$rows_json" \
  '
  {
    version: "0.1",
    endpoints: (
      ($rows | map(.endpoint) | unique) as $eps
      | [
          $eps[] as $ep
          | ($rows
              | map(select(.endpoint == $ep))
              | sort_by((.kneeAtTargetRps // 0), (.achievedRatioMin // 0))
              | reverse
            ) as $compared
          | {
              endpoint: $ep,
              compared: $compared,
              leader: ($compared[0] // null)
            }
        ]
    )
  }
  | . as $matrix
  | .summary = {
      endpointCount: ($matrix.endpoints | length),
      implementationCount: (
        [ $matrix.endpoints[].compared[].impl ] | unique | length
      ),
      kneeDetectedCount: (
        [ $matrix.endpoints[].compared[] | select(.kneeDetected == true) ] | length
      )
    }
  ' > "$out_path"

echo "wrote $out_path"
