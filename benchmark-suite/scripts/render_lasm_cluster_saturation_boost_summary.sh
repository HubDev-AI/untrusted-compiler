#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
  echo "usage: $0 <saturation_boost_matrix.json> <saturation_boost_analysis.json> <out_summary.md> [recommended_verify_probe.json]" >&2
  exit 2
fi

matrix_path="$1"
analysis_path="$2"
out_path="$3"
verify_path="${4:-}"

if [ ! -f "${matrix_path}" ]; then
  echo "saturation boost matrix file not found: ${matrix_path}" >&2
  exit 2
fi
if [ ! -f "${analysis_path}" ]; then
  echo "saturation boost analysis file not found: ${analysis_path}" >&2
  exit 2
fi
if [ -n "${verify_path}" ] && [ ! -f "${verify_path}" ]; then
  echo "recommended verification file not found: ${verify_path}" >&2
  exit 2
fi

if ! jq -e '.runs | type == "array" and length > 0' "${matrix_path}" >/dev/null; then
  echo "saturation boost matrix has no runs: ${matrix_path}" >&2
  exit 2
fi

recommended_step="$(jq -r '.summary.recommendedBoostStep // empty' "${analysis_path}")"
if [ -z "${recommended_step}" ]; then
  echo "saturation boost analysis missing summary.recommendedBoostStep: ${analysis_path}" >&2
  exit 2
fi
if ! [[ "${recommended_step}" =~ ^[0-9]+$ ]]; then
  echo "saturation boost analysis has invalid recommended boost step: ${recommended_step}" >&2
  exit 2
fi
if ! jq -e --argjson step "${recommended_step}" '.runs | any(.saturationBoostStep == $step)' "${matrix_path}" >/dev/null; then
  echo "recommended boost step is not present in matrix runs: ${recommended_step}" >&2
  exit 2
fi

if [ -n "${verify_path}" ]; then
  verify_step="$(jq -r '.run.autoscaleSaturationBoostStep // empty' "${verify_path}")"
  if [ -z "${verify_step}" ]; then
    echo "recommended verification file missing run.autoscaleSaturationBoostStep: ${verify_path}" >&2
    exit 2
  fi
  if [ "${verify_step}" != "${recommended_step}" ]; then
    echo "recommended verification boost step mismatch: expected ${recommended_step}, got ${verify_step}" >&2
    exit 2
  fi
fi

mkdir -p "$(dirname "${out_path}")"

now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
run_count="$(jq -r '.summary.runCount // 0' "${analysis_path}")"
pass_count="$(jq -r '.summary.passCount // 0' "${analysis_path}")"
selection_mode="$(jq -r '.summary.selectionMode // "unknown"' "${analysis_path}")"

{
  echo "# LASM Saturation Boost Summary (v0.1)"
  echo
  echo "- Generated UTC: ${now_utc}"
  echo "- Matrix source: ${matrix_path}"
  echo "- Analysis source: ${analysis_path}"
  if [ -n "${verify_path}" ]; then
    echo "- Verification source: ${verify_path}"
  fi
  echo "- Run count: ${run_count}"
  echo "- Pass runs: ${pass_count}"
  echo "- Selection mode: ${selection_mode}"
  echo "- Recommended boost step: ${recommended_step}"
  echo

  echo "## Probe Profile"
  echo
  echo "- Project path: $(jq -r '.run.projectPath // "unknown"' "${matrix_path}")"
  echo "- Request path: $(jq -r '.run.requestPath // "unknown"' "${matrix_path}")"
  echo "- Duration: $(jq -r '.run.duration // "unknown"' "${matrix_path}")"
  echo "- Threads: $(jq -r '.run.threads // "unknown"' "${matrix_path}")"
  echo "- Connections: $(jq -r '.run.connections // "unknown"' "${matrix_path}")"
  echo "- Target requests: $(jq -r '.run.targetRequests // "unknown"' "${matrix_path}")"
  echo

  echo "## Ranked Runs"
  echo
  echo "| Rank | Boost Step | Pass | Requests/sec | Requests | Peak RSS (KB) |"
  echo "|---:|---:|---|---:|---:|---:|"
  jq -r '
    .rankedRuns
    | to_entries[]
    | [
        (.key + 1),
        .value.saturationBoostStep,
        (.value.pass | tostring),
        (.value.requestsPerSec | tostring),
        (.value.requests | tostring),
        (.value.peakRssKb | tostring)
      ]
    | @tsv
  ' "${analysis_path}" \
    | while IFS=$'\t' read -r rank step pass reqps requests rss; do
        printf "| %s | %s | %s | %s | %s | %s |\n" "$rank" "$step" "$pass" "$reqps" "$requests" "$rss"
      done
  echo

  echo "## Recommendation"
  echo
  jq -r --arg step "${recommended_step}" '
    .rankedRuns[]
    | select((.saturationBoostStep | tostring) == $step)
    | "- selected row: pass=\(.pass), requestsPerSec=\(.requestsPerSec), requests=\(.requests), peakRssKb=\(.peakRssKb)"
  ' "${analysis_path}"
  echo

  if [ -n "${verify_path}" ]; then
    echo "## Recommended Step Verification"
    echo
    echo "- Boost step: $(jq -r '.run.autoscaleSaturationBoostStep // "unknown"' "${verify_path}")"
    echo "- Pass: $(jq -r '.pass // "unknown"' "${verify_path}")"
    echo "- Requests target met: $(jq -r '.requestsTargetMet // "unknown"' "${verify_path}")"
    echo "- Requests: $(jq -r '.observed.requests // "unknown"' "${verify_path}")"
    echo "- Requests/sec: $(jq -r '.observed.requestsPerSec // "unknown"' "${verify_path}")"
    echo "- Peak RSS (KB): $(jq -r '.observed.peakRssKb // "unknown"' "${verify_path}")"
    echo "- p99: $(jq -r '.observed.p99 // "unknown"' "${verify_path}")"
    echo
  fi
} > "${out_path}"

echo "wrote ${out_path}"
