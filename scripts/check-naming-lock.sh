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
  local contract_validator="benchmark-suite/scripts/validate_contract_schema.sh"

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
require_contract_token "sec4 replay" "sec4 replay"
require_contract_token "\\.ut\\b" ".ut"
require_contract_token "ut/std" "ut/std"
require_contract_token "ut/http" "ut/http"
require_contract_token "ut/sec" "ut/sec"
require_contract_token "path_suffixes\\s*=\\s*\\[\\s*\"ut\"\\s*\\]" "zed path_suffixes=[\"ut\"]"
check_benchmark_impl_contract
check_benchmark_artifact_contract
check_benchmark_contract_spec

echo "ok: naming lock check passed"
