#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
samples_dir="${root_dir}/scripts/testdata"
schema_dir="${root_dir}/spec/schemas"

usage() {
  cat <<'USAGE'
usage: validate_contract_schema.sh [--samples-dir <dir>] [--schema-dir <dir>]

Validates benchmark artifact sample JSON files against machine schema assets
(required top-level keys, version constants, and artifact item keys).
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --samples-dir)
      samples_dir="$2"
      shift 2
      ;;
    --schema-dir)
      schema_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "error: missing required file: ${path}" >&2
    return 1
  fi
}

validate_json_required_keys() {
  local sample_path="$1"
  local schema_path="$2"
  local label="$3"
  local failed=0

  require_file "${sample_path}" || return 1
  require_file "${schema_path}" || return 1

  local key
  while IFS= read -r key; do
    [[ -z "${key}" ]] && continue
    if ! jq -e --arg k "${key}" 'has($k)' "${sample_path}" >/dev/null; then
      echo "error: ${label} missing required key '${key}': ${sample_path}" >&2
      failed=1
    fi
  done < <(jq -r '.required[]?' "${schema_path}")

  local expected_version
  expected_version="$(jq -r '.properties.version.const // empty' "${schema_path}")"
  if [[ -n "${expected_version}" ]]; then
    if ! jq -e --arg v "${expected_version}" '.version == $v' "${sample_path}" >/dev/null; then
      echo "error: ${label} version mismatch (expected ${expected_version}): ${sample_path}" >&2
      failed=1
    fi
  fi

  if jq -e '.properties.artifacts.items.required? != null' "${schema_path}" >/dev/null; then
    local item_key
    while IFS= read -r item_key; do
      [[ -z "${item_key}" ]] && continue
      if ! jq -e --arg k "${item_key}" '
        (.artifacts | type == "array")
        and (all(.artifacts[]; has($k)))
      ' "${sample_path}" >/dev/null; then
        echo "error: ${label} artifact item missing key '${item_key}': ${sample_path}" >&2
        failed=1
      fi
    done < <(jq -r '.properties.artifacts.items.required[]?' "${schema_path}")
  fi

  return "${failed}"
}

validate_compare_matrix_rows() {
  local sample_path="$1"
  local label="$2"

  if ! jq -e '
    def row_ok:
      (type == "object")
      and (.impl | type == "string")
      and (.endpoint | type == "string")
      and (.targetRps | type == "number")
      and (.requestsPerSec | type == "number")
      and (.p99 | type == "string")
      and (.loadGenerator | type == "string")
      and (.constantRate | type == "boolean");
    def row_eq(a; b):
      a.impl == b.impl
      and a.endpoint == b.endpoint
      and a.targetRps == b.targetRps
      and a.requestsPerSec == b.requestsPerSec
      and a.p99 == b.p99
      and a.loadGenerator == b.loadGenerator
      and a.constantRate == b.constantRate;

    (.endpoints | type == "array" and length > 0)
    and all(.endpoints[];
      . as $entry
      | ($entry.endpoint | type == "string")
      and ($entry.compared | type == "array" and length > 0)
      and (all($entry.compared[]; row_ok and (.endpoint == $entry.endpoint)))
      and ($entry.leader | row_ok)
      and ($entry.leader.endpoint == $entry.endpoint)
      and (any($entry.compared[]; row_eq(.; $entry.leader)))
    )
  ' "${sample_path}" >/dev/null; then
    echo "error: ${label} row contract mismatch (shape, endpoint alignment, or leader membership): ${sample_path}" >&2
    return 1
  fi
}

validate_compare_report_rows() {
  local sample_path="$1"
  local label="$2"

  if ! jq -e '
    def row_ok:
      (type == "object")
      and (.impl | type == "string")
      and (.endpoint | type == "string")
      and (.targetRps | type == "number")
      and (.requestsPerSec | type == "number")
      and (.p99 | type == "string")
      and (.loadGenerator | type == "string")
      and (.constantRate | type == "boolean");
    def row_eq(a; b):
      a.impl == b.impl
      and a.endpoint == b.endpoint
      and a.targetRps == b.targetRps
      and a.requestsPerSec == b.requestsPerSec
      and a.p99 == b.p99
      and a.loadGenerator == b.loadGenerator
      and a.constantRate == b.constantRate;

    . as $report
    | ($report.endpoint | type == "string")
    and ($report.compared | type == "array" and length > 0)
    and (all($report.compared[]; row_ok and (.endpoint == $report.endpoint)))
    and ($report.leader | row_ok)
    and ($report.leader.endpoint == $report.endpoint)
    and (any($report.compared[]; row_eq(.; $report.leader)))
  ' "${sample_path}" >/dev/null; then
    echo "error: ${label} row contract mismatch (shape, endpoint alignment, or leader membership): ${sample_path}" >&2
    return 1
  fi
}

report_schema="${schema_dir}/report.schema.json"
summary_schema="${schema_dir}/summary.schema.json"
step_summary_schema="${schema_dir}/step-summary.schema.json"
compare_report_schema="${schema_dir}/compare-report.schema.json"
compare_matrix_schema="${schema_dir}/compare-matrix.schema.json"
analysis_schema="${schema_dir}/analysis.schema.json"
step_matrix_schema="${schema_dir}/step-matrix.schema.json"
artifact_manifest_schema="${schema_dir}/artifact-manifest.schema.json"

failed=0

for impl in sec4 go node rust; do
  report_sample="${samples_dir}/sample-${impl}-report.json"
  if ! validate_json_required_keys "${report_sample}" "${report_schema}" "report sample"; then
    failed=1
  fi
  if ! jq -e --arg impl "${impl}" '.impl == $impl' "${report_sample}" >/dev/null; then
    echo "error: report sample impl mismatch for ${impl}: ${report_sample}" >&2
    failed=1
  fi
done

for endpoint in ping decode; do
  summary_sample="${samples_dir}/sample-summary-${endpoint}.json"
  if ! validate_json_required_keys "${summary_sample}" "${summary_schema}" "summary sample"; then
    failed=1
  fi
done

if ! validate_json_required_keys "${samples_dir}/sample-step-summary-decode.json" "${step_summary_schema}" "step-summary sample"; then
  failed=1
fi

if ! validate_json_required_keys "${samples_dir}/sample-compare-report-ping.json" "${compare_report_schema}" "compare-report sample"; then
  failed=1
fi
if ! validate_compare_report_rows "${samples_dir}/sample-compare-report-ping.json" "compare-report sample"; then
  failed=1
fi

if ! validate_json_required_keys "${samples_dir}/sample-compare-matrix.json" "${compare_matrix_schema}" "compare-matrix sample"; then
  failed=1
fi
if ! validate_compare_matrix_rows "${samples_dir}/sample-compare-matrix.json" "compare-matrix sample"; then
  failed=1
fi

trend_compare_samples=(
  "${samples_dir}/sample-trend-compare-matrix.json"
  "${samples_dir}/sample-trend-compare-matrix-wrk.json"
)
for trend_sample in "${trend_compare_samples[@]}"; do
  if ! validate_json_required_keys "${trend_sample}" "${compare_matrix_schema}" "trend compare-matrix sample"; then
    failed=1
    continue
  fi
  if ! validate_compare_matrix_rows "${trend_sample}" "trend compare-matrix sample"; then
    failed=1
  fi
done

if ! validate_json_required_keys "${samples_dir}/sample-analysis.json" "${analysis_schema}" "analysis sample"; then
  failed=1
fi

if ! validate_json_required_keys "${samples_dir}/sample-step-matrix.json" "${step_matrix_schema}" "step-matrix sample"; then
  failed=1
fi

if ! validate_json_required_keys "${samples_dir}/sample-artifact-manifest.json" "${artifact_manifest_schema}" "artifact-manifest sample"; then
  failed=1
fi

if [[ "${failed}" -ne 0 ]]; then
  exit 1
fi

echo "benchmark contract schema validation passed"
