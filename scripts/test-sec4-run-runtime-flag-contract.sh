#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--cli <path>]

Validates sec4 run runtime-flag bridge contract in CLI source.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cli_path="${root_dir}/compiler/sec4-cli/src/main.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --cli)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      cli_path="$2"
      shift 2
      ;;
    --cli=*)
      cli_path="${1#--cli=}"
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

if [ ! -f "${cli_path}" ]; then
  echo "missing sec4 CLI source: ${cli_path}" >&2
  exit 1
fi

require_pattern() {
  local pattern="$1"
  local label="$2"
  if ! rg -q -- "${pattern}" "${cli_path}"; then
    echo "missing run-flag contract pattern (${label}) in ${cli_path}" >&2
    exit 1
  fi
}

require_pattern '^\s*port:\s*Option<u16>,\s*$' 'run port field'
require_pattern '^\s*oneshot:\s*bool,\s*$' 'run oneshot field'
require_pattern '^\s*max_body_bytes:\s*Option<u64>,\s*$' 'run max-body-bytes field'
require_pattern '^\s*serve_timeout_ms:\s*Option<u64>,\s*$' 'run serve-timeout field'
require_pattern '=> cmd_run\(' 'run dispatch invokes cmd_run'
require_pattern 'cmd\.env\("SEC4_RT_HTTP_PORT", port\.to_string\(\)\);' 'run port env bridge'
require_pattern 'cmd\.env\("SEC4_RT_HTTP_SERVE_MODE", "oneshot"\);' 'run oneshot env bridge'
require_pattern 'cmd\.env\("SEC4_RT_HTTP_MAX_BODY_BYTES", bytes\.to_string\(\)\);' 'run body-cap env bridge'
require_pattern 'cmd\.env\("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", timeout_ms\.to_string\(\)\);' 'run timeout env bridge'

echo "sec4 run runtime-flag contract test passed"
