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

echo "ok: naming lock check passed"
