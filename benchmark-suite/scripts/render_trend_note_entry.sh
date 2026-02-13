#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 <compare_matrix.json> [--endpoints ping,decode] [--baseline-dir <path>] [--date YYYY-MM-DD] [--out <path>]

Renders a markdown trend-note table from compare-matrix leader metrics.
USAGE
}

if [ "$#" -lt 1 ]; then
  usage
  exit 2
fi

matrix_path="$1"
shift

endpoints_csv="ping,decode"
baseline_dir=""
entry_date="$(date -u +%Y-%m-%d)"
out_path=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      endpoints_csv="$2"
      shift 2
      ;;
    --endpoints=*)
      endpoints_csv="${1#--endpoints=}"
      shift
      ;;
    --baseline-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      baseline_dir="$2"
      shift 2
      ;;
    --baseline-dir=*)
      baseline_dir="${1#--baseline-dir=}"
      shift
      ;;
    --date)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      entry_date="$2"
      shift 2
      ;;
    --date=*)
      entry_date="${1#--date=}"
      shift
      ;;
    --out)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_path="$2"
      shift 2
      ;;
    --out=*)
      out_path="${1#--out=}"
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

if [ -z "$baseline_dir" ]; then
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  baseline_dir="$(cd "${script_dir}/.." && pwd)/baselines"
fi

abs_passes=0
abs_fails=0
base_passes=0
base_fails=0
base_na=0

row_lines=()

default_thresholds() {
  case "$1" in
    ping) echo "30 85" ;;
    decode) echo "80 60" ;;
    *) echo "" ;;
  esac
}

for raw_endpoint in ${endpoints_csv//,/ }; do
  endpoint="${raw_endpoint// /}"
  [ -z "$endpoint" ] && continue

  leader_json="$(jq -c --arg endpoint "$endpoint" '.endpoints[] | select(.endpoint == $endpoint) | .leader' "$matrix_path" | head -n 1)"
  if [ -z "$leader_json" ] || [ "$leader_json" = "null" ]; then
    row_lines+=("| ${endpoint} | missing | - | - | missing | missing |")
    continue
  fi

  leader_impl="$(jq -r '.impl // "unknown"' <<<"$leader_json")"
  p99_ms="$(jq -r '
    (.p99 // "")
    | tostring
    | capture("(?<n>[0-9]+(\\.[0-9]+)?)")?.n // ""
  ' <<<"$leader_json")"
  if [ -z "$p99_ms" ]; then
    p99_ms="0"
  fi
  target_rps="$(jq -r '.targetRps // 0' <<<"$leader_json")"
  actual_rps="$(jq -r '.requestsPerSec // 0' <<<"$leader_json")"
  constant_rate="$(jq -r 'if has("constantRate") then .constantRate else true end' <<<"$leader_json")"
  coverage_pct="$(awk -v target="${target_rps}" -v actual="${actual_rps}" 'BEGIN { if (target + 0 > 0) printf "%.2f", (actual / target) * 100; else printf "0.00" }')"
  coverage_display="${coverage_pct}"
  p99_fmt="$(awk -v n="${p99_ms}" 'BEGIN { printf "%.2f", n + 0 }')"

  threshold_pair="$(default_thresholds "$endpoint")"
  abs_status="n/a"
  if [ "$constant_rate" != "true" ]; then
    abs_status="n/a"
    coverage_display="n/a"
  elif [ -n "$threshold_pair" ]; then
    read -r max_p99 min_cov <<<"$threshold_pair"
    if awk -v p99="${p99_ms}" -v max="${max_p99}" -v cov="${coverage_pct}" -v min="${min_cov}" 'BEGIN { exit !((p99+0 <= max+0) && (cov+0 >= min+0)) }'; then
      abs_status="pass"
      abs_passes=$((abs_passes + 1))
    else
      abs_status="fail"
      abs_fails=$((abs_fails + 1))
    fi
  fi

  baseline_status="n/a"
  baseline_path="${baseline_dir}/node-${endpoint}-trend-baseline.json"
  if [ "$constant_rate" != "true" ]; then
    baseline_status="n/a"
    base_na=$((base_na + 1))
  elif [ -f "$baseline_path" ]; then
    baseline_p99="$(jq -r '.baselineP99Ms // empty' "$baseline_path")"
    baseline_cov="$(jq -r '.baselineCoveragePct // empty' "$baseline_path")"
    max_p99_regress="$(jq -r '.maxP99RegressionPct // 20' "$baseline_path")"
    max_cov_drop="$(jq -r '.maxCoverageDropPct // 5' "$baseline_path")"
    if [ -n "$baseline_p99" ] && [ -n "$baseline_cov" ]; then
      baseline_p99_limit="$(awk -v base="${baseline_p99}" -v pct="${max_p99_regress}" 'BEGIN { printf "%.6f", base * (1 + pct / 100.0) }')"
      baseline_cov_limit="$(awk -v base="${baseline_cov}" -v pct="${max_cov_drop}" 'BEGIN { printf "%.6f", base - pct }')"
      if awk -v p99="${p99_ms}" -v p99_limit="${baseline_p99_limit}" -v cov="${coverage_pct}" -v cov_limit="${baseline_cov_limit}" 'BEGIN { exit !((p99+0 <= p99_limit+0) && (cov+0 >= cov_limit+0)) }'; then
        baseline_status="pass"
        base_passes=$((base_passes + 1))
      else
        baseline_status="fail"
        base_fails=$((base_fails + 1))
      fi
    else
      baseline_status="invalid"
      base_na=$((base_na + 1))
    fi
  else
    base_na=$((base_na + 1))
  fi

  row_lines+=("| ${endpoint} | ${leader_impl} | ${p99_fmt} | ${coverage_display} | ${abs_status} | ${baseline_status} |")
done

overall_abs="n/a"
if [ "$abs_fails" -gt 0 ]; then
  overall_abs="fail"
elif [ "$abs_passes" -gt 0 ]; then
  overall_abs="pass"
fi

overall_base="n/a"
if [ "$base_fails" -gt 0 ]; then
  overall_base="fail"
elif [ "$base_passes" -gt 0 ]; then
  overall_base="pass"
elif [ "$base_na" -gt 0 ]; then
  overall_base="n/a"
fi

{
  echo "## Trend Entry (${entry_date})"
  echo
  echo "- Source matrix: \`${matrix_path}\`"
  echo "- Endpoints: \`${endpoints_csv}\`"
  echo
  echo "| Endpoint | Leader | p99 (ms) | Coverage (%) | Absolute Guard | Baseline Guard |"
  echo "| --- | --- | ---: | ---: | --- | --- |"
  for row in "${row_lines[@]}"; do
    echo "${row}"
  done
  echo
  echo "- Overall absolute guard status: ${overall_abs}"
  echo "- Overall baseline guard status: ${overall_base}"
} > "${out_path:-/dev/stdout}"

if [ -n "${out_path}" ]; then
  echo "wrote ${out_path}"
fi
