#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--artifacts-dir <path>]

Validates runtime-smoke artifact directory contents and response contracts.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_dir="${root_dir}/build/runtime-smoke"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --artifacts-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      artifacts_dir="$2"
      shift 2
      ;;
    --artifacts-dir=*)
      artifacts_dir="${1#--artifacts-dir=}"
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

if [ ! -d "${artifacts_dir}" ]; then
  echo "runtime-smoke artifacts directory does not exist: ${artifacts_dir}" >&2
  exit 1
fi

required_files=(
  "run-metadata.txt"
  "health.headers"
  "health.body"
  "health.run.log"
  "users.headers"
  "users.body"
  "users.run.log"
)

for file in "${required_files[@]}"; do
  path="${artifacts_dir}/${file}"
  if [ ! -f "${path}" ]; then
    echo "missing runtime-smoke artifact file: ${file}" >&2
    exit 1
  fi
  if [ ! -s "${path}" ]; then
    echo "runtime-smoke artifact file is empty: ${file}" >&2
    exit 1
  fi
done

if ! rg -Fq 'HTTP/1.1 200 OK' "${artifacts_dir}/health.headers"; then
  echo "health.headers missing 200 OK status line" >&2
  exit 1
fi

if ! rg -iq '^X-Trace-Id:[[:space:]]*.+$' "${artifacts_dir}/health.headers"; then
  echo "health.headers missing X-Trace-Id header" >&2
  exit 1
fi

health_header_trace_id="$(sed -nE 's/^X-Trace-Id:[[:space:]]*//Ip' "${artifacts_dir}/health.headers" | tr -d '\r' | head -n 1)"
if [[ ! "${health_header_trace_id}" =~ ^rt-[0-9]+$ ]]; then
  echo "health.headers contains malformed X-Trace-Id value" >&2
  exit 1
fi

if ! rg -Fq 'HTTP/1.1 201 Created' "${artifacts_dir}/users.headers"; then
  echo "users.headers missing 201 Created status line" >&2
  exit 1
fi

if [ "$(cat "${artifacts_dir}/health.body")" != "ok" ]; then
  echo "health.body does not match expected 'ok' payload" >&2
  exit 1
fi

if ! rg -Fq -- '--oneshot' "${artifacts_dir}/health.run.log"; then
  echo "health.run.log missing --oneshot invocation token" >&2
  exit 1
fi

if ! jq -e '.ok == true and .status == 201 and (.traceId | type == "string" and test("^rt-[0-9]+$")) and (.timeMs | type == "number") and has("data")' "${artifacts_dir}/users.body" >/dev/null; then
  echo "users.body does not match expected std-success envelope contract" >&2
  exit 1
fi

if ! rg -iq '^X-Trace-Id:[[:space:]]*.+$' "${artifacts_dir}/users.headers"; then
  echo "users.headers missing X-Trace-Id header" >&2
  exit 1
fi

users_header_trace_id="$(sed -nE 's/^X-Trace-Id:[[:space:]]*//Ip' "${artifacts_dir}/users.headers" | tr -d '\r' | head -n 1)"
users_body_trace_id="$(jq -r '.traceId' "${artifacts_dir}/users.body")"

if [ "${users_header_trace_id}" != "${users_body_trace_id}" ]; then
  echo "users traceId mismatch between headers and body" >&2
  exit 1
fi

if ! rg -Fq -- '--oneshot' "${artifacts_dir}/users.run.log"; then
  echo "users.run.log missing --oneshot invocation token" >&2
  exit 1
fi

if ! rg -q '^port=[0-9]+$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing numeric port field" >&2
  exit 1
fi

port_value="$(sed -n 's/^port=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"

if [ "${port_value}" -lt 1 ] || [ "${port_value}" -gt 65535 ]; then
  echo "run-metadata.txt port must be between 1 and 65535" >&2
  exit 1
fi

if ! rg -Fq -- "--port ${port_value}" "${artifacts_dir}/health.run.log"; then
  echo "health.run.log missing --port ${port_value} invocation token" >&2
  exit 1
fi

if ! rg -Fq -- "--port ${port_value}" "${artifacts_dir}/users.run.log"; then
  echo "users.run.log missing --port ${port_value} invocation token" >&2
  exit 1
fi

if ! rg -q '^sourceProject=.+$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing sourceProject field" >&2
  exit 1
fi

if ! rg -q '^workProject=.+$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing workProject field" >&2
  exit 1
fi

source_project="$(sed -n 's/^sourceProject=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"
work_project="$(sed -n 's/^workProject=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"

if [ "${source_project}" = "${work_project}" ]; then
  echo "run-metadata.txt sourceProject and workProject must differ" >&2
  exit 1
fi

if ! rg -q '^oneshot=true$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing oneshot=true field" >&2
  exit 1
fi

if ! rg -q '^serveTimeoutMs=[0-9]+$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing numeric serveTimeoutMs field" >&2
  exit 1
fi

serve_timeout_ms="$(sed -n 's/^serveTimeoutMs=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"

if [ "${serve_timeout_ms}" -le 0 ]; then
  echo "run-metadata.txt serveTimeoutMs must be greater than 0" >&2
  exit 1
fi

if ! rg -Fq -- "--serve-timeout-ms ${serve_timeout_ms}" "${artifacts_dir}/health.run.log"; then
  echo "health.run.log missing --serve-timeout-ms ${serve_timeout_ms} invocation token" >&2
  exit 1
fi

if ! rg -Fq -- "--serve-timeout-ms ${serve_timeout_ms}" "${artifacts_dir}/users.run.log"; then
  echo "users.run.log missing --serve-timeout-ms ${serve_timeout_ms} invocation token" >&2
  exit 1
fi

if ! rg -q '^maxBodyBytes=(unset|[0-9]+)$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing or invalid maxBodyBytes field" >&2
  exit 1
fi

max_body_bytes="$(sed -n 's/^maxBodyBytes=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"

if [ "${max_body_bytes}" != "unset" ]; then
  if [ "${max_body_bytes}" -le 0 ]; then
    echo "run-metadata.txt maxBodyBytes must be unset or positive integer" >&2
    exit 1
  fi

  if ! rg -Fq -- "--max-body-bytes ${max_body_bytes}" "${artifacts_dir}/health.run.log"; then
    echo "health.run.log missing --max-body-bytes ${max_body_bytes} invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- "--max-body-bytes ${max_body_bytes}" "${artifacts_dir}/users.run.log"; then
    echo "users.run.log missing --max-body-bytes ${max_body_bytes} invocation token" >&2
    exit 1
  fi
fi

if ! rg -Fq 'runFlags=--port,--oneshot,--serve-timeout-ms' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing deterministic runFlags field" >&2
  exit 1
fi

echo "runtime-smoke artifacts check passed"
