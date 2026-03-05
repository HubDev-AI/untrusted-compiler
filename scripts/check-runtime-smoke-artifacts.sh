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

if ! jq -e '.ok == true and (.userId | type == "string" and test("^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89aAbB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"))' "${artifacts_dir}/users.body" >/dev/null; then
  echo "users.body does not match expected create-user success contract" >&2
  exit 1
fi

if ! rg -iq '^X-Trace-Id:[[:space:]]*.+$' "${artifacts_dir}/users.headers"; then
  echo "users.headers missing X-Trace-Id header" >&2
  exit 1
fi

users_header_trace_id="$(sed -nE 's/^X-Trace-Id:[[:space:]]*//Ip' "${artifacts_dir}/users.headers" | tr -d '\r' | head -n 1)"
if [[ ! "${users_header_trace_id}" =~ ^rt-[0-9]+$ ]]; then
  echo "users.headers contains malformed X-Trace-Id value" >&2
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

if ! rg -q '^runFlags=.+$' "${artifacts_dir}/run-metadata.txt"; then
  echo "run-metadata.txt missing deterministic runFlags field" >&2
  exit 1
fi

run_flags_value="$(sed -n 's/^runFlags=//p' "${artifacts_dir}/run-metadata.txt" | head -n 1)"
expected_run_flags="--port,--oneshot,--serve-timeout-ms"
if [ "${max_body_bytes}" != "unset" ]; then
  expected_run_flags="${expected_run_flags},--max-body-bytes"
fi

if [ "${run_flags_value}" != "${expected_run_flags}" ]; then
  echo "run-metadata.txt runFlags field does not match expected runtime flag shape" >&2
  exit 1
fi

echo "runtime-smoke artifacts check passed"
