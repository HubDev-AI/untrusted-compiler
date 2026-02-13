#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <reports_dir> <endpoint> <out_compare.json>" >&2
  exit 2
fi

reports_dir="$1"
endpoint="$2"
out="$3"

mapfile -t report_files < <(find "$reports_dir" -maxdepth 1 -type f -name '*-report.json' | sort)
if [ "${#report_files[@]}" -eq 0 ]; then
  echo "no report files found in ${reports_dir}" >&2
  exit 2
fi

rows='[]'
for report in "${report_files[@]}"; do
  impl="$(jq -r '.impl // empty' "$report")"
  if [ -z "$impl" ]; then
    continue
  fi

  summary="$(jq -c --arg endpoint "$endpoint" '.summaries[] | select(.endpoint == $endpoint)' "$report" | head -n1)"
  if [ -z "$summary" ]; then
    continue
  fi

  reqps="$(jq -r '.requestsPerSec // 0' <<<"$summary")"
  p99="$(jq -r '.latency.p99 // ""' <<<"$summary")"
  target_rps="$(jq -r '.targetRps // 0' <<<"$summary")"

  row="$(jq -n \
    --arg impl "$impl" \
    --arg p99 "$p99" \
    --argjson reqps "$reqps" \
    --argjson target "$target_rps" \
    '{impl:$impl,targetRps:$target,requestsPerSec:$reqps,p99:$p99}')"
  rows="$(jq -c --argjson row "$row" '. + [$row]' <<<"$rows")"
done

if [ "$(jq 'length' <<<"$rows")" -eq 0 ]; then
  echo "no matching endpoint summaries found for endpoint=${endpoint}" >&2
  exit 2
fi

rows_sorted="$(jq -c 'sort_by(.requestsPerSec) | reverse' <<<"$rows")"
leader="$(jq -c '.[0]' <<<"$rows_sorted")"

mkdir -p "$(dirname "$out")"

jq -n \
  --arg endpoint "$endpoint" \
  --argjson rows "$rows_sorted" \
  --argjson leader "$leader" \
  '{
    version: "0.1",
    endpoint: $endpoint,
    compared: $rows,
    leader: $leader
  }' > "$out"

echo "wrote $out"
