#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workflow_path="${root_dir}/.github/workflows/benchmark-trend.yml"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --workflow)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --workflow" >&2
        exit 2
      fi
      workflow_path="$2"
      shift 2
      ;;
    --workflow=*)
      workflow_path="${1#--workflow=}"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "${workflow_path}" ]]; then
  echo "missing workflow file: ${workflow_path}" >&2
  exit 1
fi

require_token() {
  local token="$1"
  if ! grep -q -- "${token}" "${workflow_path}"; then
    echo "missing workflow contract token '${token}' in ${workflow_path}" >&2
    exit 1
  fi
}

extract_flag_values() {
  local flag="$1"
  grep -E -- "${flag}(=|[[:space:]])" "${workflow_path}" \
    | sed -E "s/.*${flag}(=|[[:space:]]+)//" \
    | sed -E 's/[[:space:]\\]+$//' \
    | sed -E "s/^['\"]|['\"]$//g"
}

normalize_endpoint_csv() {
  local raw="$1"
  printf '%s\n' "${raw}" \
    | tr ',' '\n' \
    | sed -E 's/^[[:space:]]+|[[:space:]]+$//g' \
    | sed '/^$/d' \
    | tr '[:upper:]' '[:lower:]' \
    | sort -u \
    | paste -sd ',' -
}

require_token 'schedule:'
require_token 'cron:'
require_token 'benchmark-suite/scripts/run_full_benchmark_suite.sh'
require_token 'benchmark-suite/scripts/render_trend_note_entry.sh'
require_token 'scripts/check-benchmark-evidence-quality.sh'
require_token '--fail-on-warning'
require_token 'benchmark-suite/scripts/check_regression_thresholds.sh'
require_token '--endpoints'
require_token '--endpoint'
require_token '--max-rss-kb'
require_token 'uses: actions/upload-artifact@v4'
require_token 'name: benchmark-trend-'
require_token 'path: benchmark-suite/results'

mapfile -t endpoints_csv_values < <(extract_flag_values '--endpoints')
if [ "${#endpoints_csv_values[@]}" -lt 2 ]; then
  echo "workflow must declare --endpoints for benchmark run and trend rendering" >&2
  exit 1
fi

canonical_endpoints_csv="$(normalize_endpoint_csv "${endpoints_csv_values[0]}")"
if [ -z "${canonical_endpoints_csv}" ]; then
  echo "workflow --endpoints contract must include at least one endpoint" >&2
  exit 1
fi

for endpoints_csv in "${endpoints_csv_values[@]}"; do
  normalized_csv="$(normalize_endpoint_csv "${endpoints_csv}")"
  if [ "${normalized_csv}" != "${canonical_endpoints_csv}" ]; then
    echo "workflow endpoint-set mismatch: expected ${canonical_endpoints_csv}, found ${normalized_csv}" >&2
    exit 1
  fi
done

mapfile -t threshold_endpoints < <(extract_flag_values '--endpoint')
if [ "${#threshold_endpoints[@]}" -eq 0 ]; then
  echo "workflow must include at least one regression threshold endpoint check" >&2
  exit 1
fi

mapfile -t rss_threshold_values < <(extract_flag_values '--max-rss-kb')
if [ "${#rss_threshold_values[@]}" -ne "${#threshold_endpoints[@]}" ]; then
  echo "workflow RSS guard mismatch: expected ${#threshold_endpoints[@]} --max-rss-kb flags, found ${#rss_threshold_values[@]}" >&2
  exit 1
fi

threshold_endpoints_csv="$(normalize_endpoint_csv "$(printf '%s,' "${threshold_endpoints[@]}")")"
if [ "${threshold_endpoints_csv}" != "${canonical_endpoints_csv}" ]; then
  echo "workflow threshold endpoint-set mismatch: expected ${canonical_endpoints_csv}, found ${threshold_endpoints_csv}" >&2
  exit 1
fi

echo "benchmark trend workflow contract test passed"
