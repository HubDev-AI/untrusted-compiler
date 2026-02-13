#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cli_path="${root_dir}/compiler/sec4-cli/src/main.rs"

if [[ ! -f "${cli_path}" ]]; then
  echo "missing sec4 CLI source: ${cli_path}" >&2
  exit 1
fi

require_pattern() {
  local pattern="$1"
  local label="$2"
  if ! rg -q -- "${pattern}" "${cli_path}"; then
    echo "missing CLI command contract pattern (${label}) in ${cli_path}" >&2
    exit 1
  fi
}

forbid_pattern() {
  local pattern="$1"
  local label="$2"
  if rg -q -- "${pattern}" "${cli_path}"; then
    echo "forbidden CLI command contract pattern (${label}) in ${cli_path}" >&2
    exit 1
  fi
}

require_pattern '^\s*Audit\(AuditArgs\),\s*$' 'top-level audit subcommand'
require_pattern '^\s*Gate \{\s*$' 'top-level gate subcommand'
require_pattern '^\s*Explain \{\s*$' 'top-level explain subcommand'
require_pattern 'Some\(fail_on\.as_deref\(\)\.unwrap_or\("risk>=HIGH"\)\)' 'gate default fail threshold'

forbid_pattern '^\s*Sec\s*\{' 'legacy nested sec subcommand alias'
forbid_pattern 'Commands::Sec\b' 'legacy nested sec dispatch path'

echo "sec4 CLI command contract test passed"
