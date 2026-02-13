#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

SEARCH_PATHS=(
  "README.md"
  "docs"
  "compiler"
  "runtime"
  "examples"
  "benchmark-suite"
  "scripts"
  "zed-extension"
  "tree-sitter-untrusted"
)

LEGACY_PATTERNS=(
  "AI""Lang"
  "AI"" Lang"
  "ai""-lang"
  "ailang-""language-server"
  "tree-sitter-""ailang"
  "zed-""ailang"
  "sec4 sec ""audit"
  "\\.a""i\\b"
)

check_legacy_patterns() {
  local failed=0
  local pattern
  for pattern in "${LEGACY_PATTERNS[@]}"; do
    local matches
    matches="$(
      rg -n --hidden \
        --glob '!.git/**' \
        --glob '!target/**' \
        --glob '!scripts/check-naming-lock.sh' \
        -e "${pattern}" \
        "${SEARCH_PATHS[@]}" || true
    )"
    if [[ -n "${matches}" ]]; then
      echo "error: found legacy naming pattern '${pattern}':" >&2
      echo "${matches}" >&2
      failed=1
    fi
  done
  return "${failed}"
}

require_contract_token() {
  local pattern="$1"
  local label="$2"
  if ! rg -q --hidden --glob '!.git/**' --glob '!target/**' -e "${pattern}" "${SEARCH_PATHS[@]}"; then
    echo "error: missing required naming contract token: ${label}" >&2
    return 1
  fi
}

validate_json_required_keys() {
  local sample_path="$1"
  local schema_path="$2"
  local label="$3"
  local failed=0

  if [[ ! -f "${schema_path}" ]]; then
    echo "error: missing schema asset: ${schema_path}" >&2
    return 1
  fi

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

check_benchmark_impl_contract() {
  local failed=0
  local impl_root="benchmark-suite/services"
  local required_impls=("sec4" "go" "node" "rust" "c")
  local required

  for required in "${required_impls[@]}"; do
    if [[ ! -d "${impl_root}/${required}" ]]; then
      echo "error: missing benchmark implementation directory: ${impl_root}/${required}" >&2
      failed=1
    fi
  done

  if [[ -d "${impl_root}/ai""lang" ]]; then
    echo "error: legacy benchmark implementation directory detected: ${impl_root}/ai""lang" >&2
    failed=1
  fi

  local sample_values
  sample_values="$(
    rg -n --no-filename '"impl"[[:space:]]*:' benchmark-suite/scripts/testdata/*.json 2>/dev/null \
      | awk -F'"' '{print $4}' \
      | sort -u || true
  )"

  local impl_value
  for impl_value in ${sample_values}; do
    case "${impl_value}" in
      sec4|go|node|rust|c)
        ;;
      *)
        echo "error: benchmark testdata uses unsupported impl id: ${impl_value}" >&2
        failed=1
        ;;
    esac
  done

  return "${failed}"
}

check_benchmark_artifact_contract() {
  local failed=0
  local schema_root="benchmark-suite/spec/schemas"
  local contract_validator="benchmark-suite/scripts/validate_contract_schema.sh"
  local report_schema="${schema_root}/report.schema.json"
  local summary_schema="${schema_root}/summary.schema.json"
  local step_summary_schema="${schema_root}/step-summary.schema.json"
  local compare_report_schema="${schema_root}/compare-report.schema.json"
  local step_matrix_schema="${schema_root}/step-matrix.schema.json"
  local compare_matrix_schema="${schema_root}/compare-matrix.schema.json"
  local analysis_schema="${schema_root}/analysis.schema.json"
  local artifact_manifest_schema="${schema_root}/artifact-manifest.schema.json"

  local benchmark_script_patterns=(
    "\\$\\{impl\\}-report\\.json"
    "\\$\\{impl\\}-\\$\\{endpoint\\}\\.json"
    "compare-matrix\\.json"
    "analysis\\.json"
    "step-matrix\\.json"
    "artifact-manifest\\.json"
    "benchmark-report\\.md"
    "\\$\\{impl\\}-service\\.log"
  )

  if [[ ! -x "${contract_validator}" ]]; then
    echo "error: missing executable benchmark contract validator: ${contract_validator}" >&2
    failed=1
  elif ! "${contract_validator}" >/dev/null; then
    echo "error: benchmark contract validator failed: ${contract_validator}" >&2
    failed=1
  fi

  local pattern
  for pattern in "${benchmark_script_patterns[@]}"; do
    if ! rg -q --hidden --glob '!.git/**' --glob '!target/**' -e "${pattern}" benchmark-suite/scripts; then
      echo "error: benchmark scripts missing artifact pattern: ${pattern}" >&2
      failed=1
    fi
  done

  local required_report_samples=(
    "sample-sec4-report.json"
    "sample-go-report.json"
    "sample-node-report.json"
    "sample-rust-report.json"
  )

  local sample
  for sample in "${required_report_samples[@]}"; do
    local path="benchmark-suite/scripts/testdata/${sample}"
    if [[ ! -f "${path}" ]]; then
      echo "error: missing benchmark report sample: ${path}" >&2
      failed=1
      continue
    fi

    local expected_impl="${sample#sample-}"
    expected_impl="${expected_impl%-report.json}"
    if ! jq -e --arg impl "${expected_impl}" '
      (.impl == $impl)
    ' "${path}" >/dev/null; then
      echo "error: benchmark report sample schema mismatch: ${path}" >&2
      failed=1
    fi
    if ! validate_json_required_keys "${path}" "${report_schema}" "benchmark report sample"; then
      failed=1
    fi
  done

  local summary_samples=(
    "benchmark-suite/scripts/testdata/sample-summary-ping.json"
    "benchmark-suite/scripts/testdata/sample-summary-decode.json"
  )
  for sample in "${summary_samples[@]}"; do
    if [[ ! -f "${sample}" ]]; then
      echo "error: missing benchmark summary sample: ${sample}" >&2
      failed=1
      continue
    fi
    if ! validate_json_required_keys "${sample}" "${summary_schema}" "benchmark summary sample"; then
      echo "error: benchmark summary sample schema mismatch: ${sample}" >&2
      failed=1
    fi
  done

  local step_summary_sample="benchmark-suite/scripts/testdata/sample-step-summary-decode.json"
  if [[ ! -f "${step_summary_sample}" ]]; then
    echo "error: missing benchmark step-summary sample: ${step_summary_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${step_summary_sample}" "${step_summary_schema}" "benchmark step-summary sample"; then
    echo "error: benchmark step-summary sample schema mismatch: ${step_summary_sample}" >&2
    failed=1
  fi

  local compare_report_sample="benchmark-suite/scripts/testdata/sample-compare-report-ping.json"
  if [[ ! -f "${compare_report_sample}" ]]; then
    echo "error: missing benchmark compare-report sample: ${compare_report_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${compare_report_sample}" "${compare_report_schema}" "benchmark compare-report sample"; then
    echo "error: benchmark compare-report sample schema mismatch: ${compare_report_sample}" >&2
    failed=1
  elif ! validate_compare_report_rows "${compare_report_sample}" "benchmark compare-report sample"; then
    echo "error: benchmark compare-report sample row-shape mismatch: ${compare_report_sample}" >&2
    failed=1
  fi

  local step_matrix_sample="benchmark-suite/scripts/testdata/sample-step-matrix.json"
  if [[ ! -f "${step_matrix_sample}" ]]; then
    echo "error: missing benchmark step-matrix sample: ${step_matrix_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${step_matrix_sample}" "${step_matrix_schema}" "benchmark step-matrix sample"; then
    echo "error: benchmark step-matrix sample schema mismatch: ${step_matrix_sample}" >&2
    failed=1
  fi

  local compare_matrix_sample="benchmark-suite/scripts/testdata/sample-compare-matrix.json"
  if [[ ! -f "${compare_matrix_sample}" ]]; then
    echo "error: missing benchmark compare-matrix sample: ${compare_matrix_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${compare_matrix_sample}" "${compare_matrix_schema}" "benchmark compare-matrix sample"; then
    echo "error: benchmark compare-matrix sample schema mismatch: ${compare_matrix_sample}" >&2
    failed=1
  elif ! validate_compare_matrix_rows "${compare_matrix_sample}" "benchmark compare-matrix sample"; then
    echo "error: benchmark compare-matrix sample row-shape mismatch: ${compare_matrix_sample}" >&2
    failed=1
  fi

  local trend_compare_samples=(
    "benchmark-suite/scripts/testdata/sample-trend-compare-matrix.json"
    "benchmark-suite/scripts/testdata/sample-trend-compare-matrix-wrk.json"
  )
  local trend_compare_sample
  for trend_compare_sample in "${trend_compare_samples[@]}"; do
    if [[ ! -f "${trend_compare_sample}" ]]; then
      echo "error: missing benchmark trend compare-matrix sample: ${trend_compare_sample}" >&2
      failed=1
      continue
    fi
    if ! validate_json_required_keys "${trend_compare_sample}" "${compare_matrix_schema}" "benchmark trend compare-matrix sample"; then
      echo "error: benchmark trend compare-matrix sample schema mismatch: ${trend_compare_sample}" >&2
      failed=1
      continue
    fi
    if ! validate_compare_matrix_rows "${trend_compare_sample}" "benchmark trend compare-matrix sample"; then
      echo "error: benchmark trend compare-matrix sample row-shape mismatch: ${trend_compare_sample}" >&2
      failed=1
    fi
  done

  local analysis_sample="benchmark-suite/scripts/testdata/sample-analysis.json"
  if [[ ! -f "${analysis_sample}" ]]; then
    echo "error: missing benchmark analysis sample: ${analysis_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${analysis_sample}" "${analysis_schema}" "benchmark analysis sample"; then
    echo "error: benchmark analysis sample schema mismatch: ${analysis_sample}" >&2
    failed=1
  fi

  local manifest_sample="benchmark-suite/scripts/testdata/sample-artifact-manifest.json"
  if [[ ! -f "${manifest_sample}" ]]; then
    echo "error: missing benchmark artifact-manifest sample: ${manifest_sample}" >&2
    failed=1
  elif ! validate_json_required_keys "${manifest_sample}" "${artifact_manifest_schema}" "benchmark artifact-manifest sample"; then
    echo "error: benchmark artifact-manifest sample schema mismatch: ${manifest_sample}" >&2
    failed=1
  fi

  return "${failed}"
}

check_benchmark_contract_spec() {
  local failed=0
  local contract="benchmark-suite/spec/artifact-contract-v0.1.md"

  if [[ ! -f "${contract}" ]]; then
    echo "error: missing benchmark artifact contract spec: ${contract}" >&2
    return 1
  fi

  local required_tokens=(
    "Benchmark Artifact Contract v0.1"
    "<impl>-report.json"
    "compare-matrix.json"
    "loadGenerator"
    "constantRate"
    "analysis.json"
    "step-matrix.json"
    "artifact-manifest.json"
    "<impl>-service.log"
    "\"version\": \"0.1\""
    "selectedEndpoints"
    "spec/schemas/report.schema.json"
    "spec/schemas/summary.schema.json"
    "spec/schemas/step-summary.schema.json"
    "spec/schemas/compare-report.schema.json"
    "spec/schemas/compare-matrix.schema.json"
    "spec/schemas/analysis.schema.json"
    "spec/schemas/step-matrix.schema.json"
    "spec/schemas/artifact-manifest.schema.json"
  )

  local token
  for token in "${required_tokens[@]}"; do
    if ! rg -Fq -- "${token}" "${contract}"; then
      echo "error: benchmark artifact contract spec missing token: ${token}" >&2
      failed=1
    fi
  done

  return "${failed}"
}

check_legacy_patterns
require_contract_token "Untrusted<T>" "Untrusted<T>"
require_contract_token "sec4 audit" "sec4 audit"
require_contract_token "sec4 explain" "sec4 explain"
require_contract_token "sec4 gate" "sec4 gate"
require_contract_token "\\.ut\\b" ".ut"
require_contract_token "ut/std" "ut/std"
require_contract_token "ut/http" "ut/http"
require_contract_token "ut/sec" "ut/sec"
require_contract_token "path_suffixes\\s*=\\s*\\[\\s*\"ut\"\\s*\\]" "zed path_suffixes=[\"ut\"]"
check_benchmark_impl_contract
check_benchmark_artifact_contract
check_benchmark_contract_spec

echo "ok: naming lock check passed"
