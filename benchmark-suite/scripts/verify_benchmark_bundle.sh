#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--skip-hash-check] <results_dir> [impls_csv] [endpoints_csv]" >&2
}

skip_hash_check="false"
if [ "${1:-}" = "--skip-hash-check" ]; then
  skip_hash_check="true"
  shift
fi

if [ "$#" -lt 1 ] || [ "$#" -gt 3 ]; then
  usage
  exit 2
fi

results_dir="$1"
impls_csv="${2:-sec4,node,go,rust}"
endpoints_csv="${3:-ping,decode,users-post,users-get}"

summaries_dir="${results_dir}/summaries"
manifest_path="${results_dir}/artifact-manifest.json"
report_path="${results_dir}/benchmark-report.md"
matrix_path="${summaries_dir}/compare-matrix.json"
analysis_path="${summaries_dir}/analysis.json"
step_matrix_path="${summaries_dir}/step-matrix.json"

if [ ! -d "$results_dir" ]; then
  echo "results directory not found: ${results_dir}" >&2
  exit 2
fi
if [ ! -d "$summaries_dir" ]; then
  echo "summaries directory not found: ${summaries_dir}" >&2
  exit 2
fi

require_file() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "missing required file: ${file}" >&2
    exit 1
  fi
}

require_json() {
  local file="$1"
  if ! jq -e '.' "$file" >/dev/null 2>&1; then
    echo "invalid json file: ${file}" >&2
    exit 1
  fi
}

verify_manifest_hash() {
  local file="$1"
  local rel="${file#$results_dir/}"

  if [ "$skip_hash_check" = "true" ]; then
    return 0
  fi

  expected_hash="$(jq -r --arg path "$rel" '.artifacts[]? | select(.path == $path) | .sha256' "$manifest_path" | head -n1)"
  if [ -z "$expected_hash" ] || [ "$expected_hash" = "null" ]; then
    echo "manifest missing artifact hash entry: ${rel}" >&2
    exit 1
  fi

  actual_hash="$(shasum -a 256 "$file" | awk '{print $1}')"
  if [ "$expected_hash" != "$actual_hash" ]; then
    echo "artifact hash mismatch for ${rel}" >&2
    exit 1
  fi
}

require_file "$manifest_path"
require_file "$report_path"
require_file "$matrix_path"
require_file "$analysis_path"
require_file "$step_matrix_path"

require_json "$manifest_path"
require_json "$matrix_path"
require_json "$analysis_path"
require_json "$step_matrix_path"

if [ "$skip_hash_check" != "true" ] && ! command -v shasum >/dev/null 2>&1; then
  echo "shasum is required for manifest hash verification" >&2
  exit 127
fi

verify_manifest_hash "$report_path"
verify_manifest_hash "$matrix_path"
verify_manifest_hash "$analysis_path"
verify_manifest_hash "$step_matrix_path"

IFS=',' read -r -a impls <<< "$impls_csv"
IFS=',' read -r -a endpoints <<< "$endpoints_csv"

for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue

  require_file "${summaries_dir}/${impl}-report.json"
  require_json "${summaries_dir}/${impl}-report.json"
  verify_manifest_hash "${summaries_dir}/${impl}-report.json"

  for raw_endpoint in "${endpoints[@]}"; do
    endpoint="${raw_endpoint// /}"
    [ -z "$endpoint" ] && continue

    require_file "${summaries_dir}/${impl}-${endpoint}.json"
    require_json "${summaries_dir}/${impl}-${endpoint}.json"
    verify_manifest_hash "${summaries_dir}/${impl}-${endpoint}.json"

    require_file "${summaries_dir}/${impl}-${endpoint}-step.json"
    require_json "${summaries_dir}/${impl}-${endpoint}-step.json"
    verify_manifest_hash "${summaries_dir}/${impl}-${endpoint}-step.json"

    require_file "${summaries_dir}/${impl}-${endpoint}-step-analysis.json"
    require_json "${summaries_dir}/${impl}-${endpoint}-step-analysis.json"
    verify_manifest_hash "${summaries_dir}/${impl}-${endpoint}-step-analysis.json"
  done
done

if ! jq -e '.endpoints | length > 0' "$matrix_path" >/dev/null; then
  echo "compare matrix has no endpoints: ${matrix_path}" >&2
  exit 1
fi
if ! jq -e '.summary.endpointCount > 0' "$analysis_path" >/dev/null; then
  echo "analysis summary has no endpoints: ${analysis_path}" >&2
  exit 1
fi
if ! jq -e '.summary.endpointCount > 0' "$step_matrix_path" >/dev/null; then
  echo "step matrix summary has no endpoints: ${step_matrix_path}" >&2
  exit 1
fi
if ! grep -q '^# Benchmark Comparative Report (v0.1)$' "$report_path"; then
  echo "benchmark report missing title: ${report_path}" >&2
  exit 1
fi

echo "benchmark bundle verified for impls=${impls_csv} endpoints=${endpoints_csv}"
