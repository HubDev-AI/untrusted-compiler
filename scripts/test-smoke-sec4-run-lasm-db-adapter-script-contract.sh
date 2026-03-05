#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--script <path>]

Validates the LASM DB adapter smoke script contract for sec4 run coverage.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script_path="${root_dir}/scripts/smoke-sec4-run-lasm-db-adapter.sh"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --script)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      script_path="$2"
      shift 2
      ;;
    --script=*)
      script_path="${1#--script=}"
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

if [ ! -f "${script_path}" ]; then
  echo "missing smoke script: ${script_path}" >&2
  exit 1
fi

if [ ! -x "${script_path}" ]; then
  echo "smoke script is not executable: ${script_path}" >&2
  exit 1
fi

required_tokens=(
  "--db-adapter"
  '--db-base'
  'db_adapter="records-log"'
  'adapter_label="records.log"'
  'dbAdapter=${db_adapter}'
  'dbAdapterLabel=${adapter_label}'
  'run_flags="--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter"'
  "--db-adapter \"\${db_adapter}\""
  "\"/db/exec?template=SELECT%201&params=alpha\""
  "\"/db/exec-tx?template=SELECT%201&params=alpha\""
  "\"/db/query-one?template=SELECT%201&params=alpha&row_schema=7\""
  "\"/db/records?includeRecords=true\""
  "default_smoke_auth_header=\"Authorization: Bearer smoke-token\""
  "SEC4_ALPHA_FULL_AUTH_HEADER"
  "auth_header=\"\${SEC4_ALPHA_FULL_AUTH_HEADER:-\$default_smoke_auth_header}\""
  ".recordId == 2"
  ".rowSchema == 7"
  'op == "execTx"'
  ".adapter == \$adapter"
  "records.log"
  "records.sqlite3"
  "jq -e --arg adapter \"\${adapter_label}\""
  "sec4 run lasm db-adapter smoke passed"
)

for token in "${required_tokens[@]}"; do
  if ! rg -Fq -- "${token}" "${script_path}"; then
    echo "missing required smoke-script token: ${token}" >&2
    exit 1
  fi
done

echo "sec4 run lasm db-adapter smoke script contract passed"
