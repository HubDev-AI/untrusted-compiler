#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--script <path>]

Validates the operator smoke script contract for sec4 run hello-api coverage.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script_path="${root_dir}/scripts/smoke-sec4-run-hello-api.sh"

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
  "--artifacts-dir"
  'serve_timeout_ms="12000"'
  'max_body_bytes=""'
  '--port "${port}"'
  "--oneshot"
  '--serve-timeout-ms "${serve_timeout_ms}"'
  '--max-body-bytes "${max_body_bytes}"'
  "oneshot=true"
  'serveTimeoutMs=${serve_timeout_ms}'
  'maxBodyBytes=${max_body_bytes:-unset}'
  'sourceProject=${source_project}'
  'workProject=${work_project}'
  "runFlags=--port,--oneshot,--serve-timeout-ms"
  "GET"
  "/health"
  "POST"
  "/users"
  "Authorization: Bearer smoke-token"
  "X-CSRF-Token: token123"
  "Cookie: csrf=token123"
  "jq -e"
  "sec4 run hello-api smoke passed"
)

for token in "${required_tokens[@]}"; do
  if ! rg -Fq -- "${token}" "${script_path}"; then
    echo "missing required smoke-script token: ${token}" >&2
    exit 1
  fi
done

echo "sec4 run hello-api smoke script contract passed"
