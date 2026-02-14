#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
lsp_source="${root_dir}/compiler/sec4-lsp/src/main.rs"

if [ ! -f "${lsp_source}" ]; then
  echo "missing LSP source file: ${lsp_source}" >&2
  exit 1
fi

require_token() {
  local token="$1"
  local label="$2"
  if ! rg -Fq -- "${token}" "${lsp_source}"; then
    echo "missing ${label} token: ${token}" >&2
    exit 1
  fi
}

require_token '"security.insert_validate_gate"' 'validate quickfix action id'
require_token '"security.insert_redact"' 'redact quickfix action id'
require_token '"effects.insert_missing_declaration"' 'effects quickfix action id'
require_token '"validate quickfix should include stable action id"' 'validate quickfix assertion'
require_token '"redact quickfix should include stable action id"' 'redact quickfix assertion'
require_token '"effect quickfix should include stable action id"' 'effects quickfix assertion'

echo "m18 editor contract expansion test passed"
