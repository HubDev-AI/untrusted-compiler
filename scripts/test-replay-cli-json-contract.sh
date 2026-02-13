#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cli_path="${root_dir}/compiler/sec4-cli/src/main.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --cli)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --cli" >&2
        exit 2
      fi
      cli_path="$2"
      shift 2
      ;;
    --cli=*)
      cli_path="${1#--cli=}"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "${cli_path}" ]]; then
  echo "missing sec4 CLI source: ${cli_path}" >&2
  exit 1
fi

require_pattern() {
  local pattern="$1"
  local label="$2"
  if ! rg -q -- "${pattern}" "${cli_path}"; then
    echo "missing replay json contract pattern (${label}) in ${cli_path}" >&2
    exit 1
  fi
}

require_literal() {
  local token="$1"
  local label="$2"
  if ! rg -Fq -- "${token}" "${cli_path}"; then
    echo "missing replay json contract token (${label}) in ${cli_path}" >&2
    exit 1
  fi
}

require_pattern 'enum ReplayOutputFormat \{' 'replay output format enum'
require_pattern 'default_value_t = ReplayOutputFormat::Text' 'replay format default text'
require_pattern '^\s*format:\s*ReplayOutputFormat,\s*$' 'replay output format field'
require_pattern 'ReplayOutputFormat::Text => \{' 'text output branch'
require_pattern 'ReplayOutputFormat::Json => \{' 'json output branch'
require_literal '"policyHashMatched"' 'json payload policy hash key'
require_literal '"compilerHashMatched"' 'json payload compiler hash key'
require_literal '"runtimeHashMatched"' 'json payload runtime hash key'
require_literal '"effectsMode"' 'json payload effects mode key'
require_literal '"warnings"' 'json payload warnings key'

echo "replay cli json contract test passed"
