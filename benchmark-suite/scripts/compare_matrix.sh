#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 <reports_dir> <out_compare_matrix.json> [impls_csv]" >&2
  exit 2
fi

reports_dir="$1"
out="$2"
impls_csv="${3:-}"

if [ -n "$impls_csv" ]; then
  report_files=()
  IFS=',' read -r -a impls <<< "$impls_csv"
  for raw_impl in "${impls[@]}"; do
    impl="${raw_impl// /}"
    [ -z "$impl" ] && continue
    candidate="${reports_dir}/${impl}-report.json"
    if [ ! -f "$candidate" ]; then
      echo "missing report file for impl=${impl}: ${candidate}" >&2
      exit 2
    fi
    report_files+=("$candidate")
  done
else
  mapfile -t report_files < <(find "$reports_dir" -maxdepth 1 -type f -name '*-report.json' | sort)
fi

if [ "${#report_files[@]}" -eq 0 ]; then
  echo "no report files found in ${reports_dir}" >&2
  exit 2
fi

rows_json="$(jq -s '
  [ .[]
    | select(.impl != null and .impl != "")
    | .impl as $impl
    | (.summaries // [])[]?
    | {
        impl: $impl,
        endpoint: .endpoint,
        targetRps: (.targetRps // 0),
        requestsPerSec: (.requestsPerSec // 0),
        p99: (.latency.p99 // "")
      }
  ]
' "${report_files[@]}")"

if [ "$(jq 'length' <<<"$rows_json")" -eq 0 ]; then
  echo "no endpoint summaries available in report files under ${reports_dir}" >&2
  exit 2
fi

mkdir -p "$(dirname "$out")"

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
              | sort_by(.requestsPerSec)
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
' > "$out"

echo "wrote $out"
