#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 <compare_matrix.json> [--endpoint ping] [--max-p99-ms 25] [--min-target-coverage 90] [--max-rss-kb 0] [--baseline path]

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
max_rss_kb=""
baseline_path=""

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
    --max-rss-kb)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      max_rss_kb="$2"
      shift 2
      ;;
    --max-rss-kb=*)
      max_rss_kb="${1#--max-rss-kb=}"
      shift
      ;;
    --baseline)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      baseline_path="$2"
      shift 2
      ;;
    --baseline=*)
      baseline_path="${1#--baseline=}"
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
if [ -n "$baseline_path" ] && [ ! -f "$baseline_path" ]; then
  echo "baseline file not found: ${baseline_path}" >&2
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
  | (try capture("(?<n>[0-9]+(\\.[0-9]+)?)").n catch null) // ""
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
leader_rss_kb="$(jq -r '
  (.rssKb // null)
  | if . == null then "" else tostring end
' <<<"$leader_json")"
if [ -n "$leader_rss_kb" ] && ! [[ "$leader_rss_kb" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  echo "leader rssKb is non-numeric for endpoint=${endpoint}" >&2
  exit 2
fi

if ! awk -v actual="$p99_ms" -v max="$max_p99_ms" 'BEGIN { exit !(actual+0 <= max+0) }'; then
  echo "threshold failure: endpoint=${endpoint} leader p99=${p99_ms}ms exceeds max=${max_p99_ms}ms" >&2
  exit 1
fi

if ! awk -v actual="$coverage_pct" -v min="$min_target_coverage" 'BEGIN { exit !(actual+0 >= min+0) }'; then
  echo "threshold failure: endpoint=${endpoint} leader coverage=${coverage_pct}% below min=${min_target_coverage}%" >&2
  exit 1
fi

if [ -n "$max_rss_kb" ]; then
  if ! [[ "$max_rss_kb" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
    echo "max-rss-kb must be numeric when provided: ${max_rss_kb}" >&2
    exit 2
  fi
  if [ -z "$leader_rss_kb" ]; then
    echo "leader rssKb is missing/non-numeric for endpoint=${endpoint}" >&2
    exit 2
  fi
  if ! awk -v actual="$leader_rss_kb" -v max="$max_rss_kb" 'BEGIN { exit !(actual+0 <= max+0) }'; then
    echo "threshold failure: endpoint=${endpoint} leader rssKb=${leader_rss_kb} exceeds max=${max_rss_kb}" >&2
    exit 1
  fi
fi

baseline_note=""
if [ -n "$baseline_path" ]; then
  baseline_endpoint="$(jq -r '.endpoint // empty' "$baseline_path")"
  if [ -z "$baseline_endpoint" ]; then
    echo "baseline file missing endpoint field: ${baseline_path}" >&2
    exit 2
  fi
  if [ "$baseline_endpoint" != "$endpoint" ]; then
    echo "baseline endpoint mismatch: expected=${endpoint} baseline=${baseline_endpoint}" >&2
    exit 2
  fi

  baseline_p99_ms="$(jq -r '.baselineP99Ms // empty' "$baseline_path")"
  baseline_coverage_pct="$(jq -r '.baselineCoveragePct // empty' "$baseline_path")"
  baseline_rss_kb="$(jq -r '.baselineRssKb // empty' "$baseline_path")"
  max_p99_regression_pct="$(jq -r '.maxP99RegressionPct // 20' "$baseline_path")"
  max_coverage_drop_pct="$(jq -r '.maxCoverageDropPct // 5' "$baseline_path")"
  max_rss_regression_pct="$(jq -r '.maxRssRegressionPct // 20' "$baseline_path")"

  if [ -z "$baseline_p99_ms" ] || [ -z "$baseline_coverage_pct" ]; then
    echo "baseline file missing required baseline metrics: ${baseline_path}" >&2
    exit 2
  fi

  baseline_p99_limit="$(awk -v base="$baseline_p99_ms" -v pct="$max_p99_regression_pct" 'BEGIN { printf "%.6f", base * (1 + pct / 100.0) }')"
  baseline_coverage_limit="$(awk -v base="$baseline_coverage_pct" -v pct="$max_coverage_drop_pct" 'BEGIN { printf "%.6f", base - pct }')"

  if ! awk -v actual="$p99_ms" -v limit="$baseline_p99_limit" 'BEGIN { exit !(actual+0 <= limit+0) }'; then
    echo "baseline regression failure: endpoint=${endpoint} leader p99=${p99_ms}ms exceeds baseline limit=${baseline_p99_limit}ms" >&2
    exit 1
  fi

  if ! awk -v actual="$coverage_pct" -v limit="$baseline_coverage_limit" 'BEGIN { exit !(actual+0 >= limit+0) }'; then
    echo "baseline regression failure: endpoint=${endpoint} leader coverage=${coverage_pct}% below baseline limit=${baseline_coverage_limit}%" >&2
    exit 1
  fi

  if [ -n "$baseline_rss_kb" ]; then
    if [ -z "$leader_rss_kb" ]; then
      echo "baseline regression failure: endpoint=${endpoint} leader rssKb is missing while baselineRssKb is set" >&2
      exit 2
    fi
    baseline_rss_limit="$(awk -v base="$baseline_rss_kb" -v pct="$max_rss_regression_pct" 'BEGIN { printf "%.6f", base * (1 + pct / 100.0) }')"
    if ! awk -v actual="$leader_rss_kb" -v limit="$baseline_rss_limit" 'BEGIN { exit !(actual+0 <= limit+0) }'; then
      echo "baseline regression failure: endpoint=${endpoint} leader rssKb=${leader_rss_kb} exceeds baseline limit=${baseline_rss_limit}" >&2
      exit 1
    fi
  fi

  baseline_note=" baseline=${baseline_path}"
fi

leader_impl="$(jq -r '.impl // "unknown"' <<<"$leader_json")"
leader_rps="$(jq -r '.requestsPerSec // 0' <<<"$leader_json")"
leader_target="$(jq -r '.targetRps // 0' <<<"$leader_json")"
rss_note=""
if [ -n "$leader_rss_kb" ]; then
  rss_note=" rssKb=${leader_rss_kb}"
fi

echo "thresholds passed: endpoint=${endpoint} leader=${leader_impl} p99=${p99_ms}ms coverage=${coverage_pct}% (${leader_rps}/${leader_target})${rss_note}${baseline_note}"
