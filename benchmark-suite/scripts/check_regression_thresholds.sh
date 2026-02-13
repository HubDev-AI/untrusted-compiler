#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 <compare_matrix.json> [--endpoint ping] [--max-p99-ms 25] [--min-target-coverage 90]

Checks benchmark leader metrics for an endpoint and fails if thresholds are exceeded.
USAGE
}

if [ "$#" -lt 1 ]; then
  usage
  exit 2
fi

matrix_path="$1"
shift

endpoint="ping"
max_p99_ms="25"
min_target_coverage="90"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --endpoint)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      endpoint="$2"
      shift 2
      ;;
    --endpoint=*)
      endpoint="${1#--endpoint=}"
      shift
      ;;
    --max-p99-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      max_p99_ms="$2"
      shift 2
      ;;
    --max-p99-ms=*)
      max_p99_ms="${1#--max-p99-ms=}"
      shift
      ;;
    --min-target-coverage)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      min_target_coverage="$2"
      shift 2
      ;;
    --min-target-coverage=*)
      min_target_coverage="${1#--min-target-coverage=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [ ! -f "$matrix_path" ]; then
  echo "compare matrix file not found: ${matrix_path}" >&2
  exit 2
fi

leader_json="$(jq -c --arg endpoint "$endpoint" '.endpoints[] | select(.endpoint == $endpoint) | .leader' "$matrix_path" | head -n 1)"
if [ -z "$leader_json" ] || [ "$leader_json" = "null" ]; then
  echo "endpoint not found in compare matrix: ${endpoint}" >&2
  exit 2
fi

p99_ms="$(jq -r '
  (.p99 // "")
  | tostring
  | capture("(?<n>[0-9]+(\\.[0-9]+)?)")?.n // ""
' <<<"$leader_json")"
if [ -z "$p99_ms" ]; then
  echo "leader p99 is missing/non-numeric for endpoint=${endpoint}" >&2
  exit 2
fi

coverage_pct="$(jq -r '
  (.targetRps // 0) as $target
  | (.requestsPerSec // 0) as $actual
  | if $target > 0 then (($actual / $target) * 100) else 0 end
' <<<"$leader_json")"

if ! awk -v actual="$p99_ms" -v max="$max_p99_ms" 'BEGIN { exit !(actual+0 <= max+0) }'; then
  echo "threshold failure: endpoint=${endpoint} leader p99=${p99_ms}ms exceeds max=${max_p99_ms}ms" >&2
  exit 1
fi

if ! awk -v actual="$coverage_pct" -v min="$min_target_coverage" 'BEGIN { exit !(actual+0 >= min+0) }'; then
  echo "threshold failure: endpoint=${endpoint} leader coverage=${coverage_pct}% below min=${min_target_coverage}%" >&2
  exit 1
fi

leader_impl="$(jq -r '.impl // "unknown"' <<<"$leader_json")"
leader_rps="$(jq -r '.requestsPerSec // 0' <<<"$leader_json")"
leader_target="$(jq -r '.targetRps // 0' <<<"$leader_json")"

echo "thresholds passed: endpoint=${endpoint} leader=${leader_impl} p99=${p99_ms}ms coverage=${coverage_pct}% (${leader_rps}/${leader_target})"
