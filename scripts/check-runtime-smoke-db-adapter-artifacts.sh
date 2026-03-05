#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--artifacts-dir <path>]

Validates runtime-smoke LASM DB-adapter artifact directory contents and persistence contracts.
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
  echo "runtime-smoke db-adapter artifacts directory does not exist: ${artifacts_dir}" >&2
  exit 1
fi

required_files=(
  "run-metadata.txt"
  "warmup.log"
  "db-exec.headers"
  "db-exec.body"
  "db-exec.run.log"
  "db-exec-tx.headers"
  "db-exec-tx.body"
  "db-exec-tx.run.log"
  "db-query-one.headers"
  "db-query-one.body"
  "db-query-one.run.log"
  "db-records.headers"
  "db-records.body"
  "db-records.run.log"
)

for file in "${required_files[@]}"; do
  path="${artifacts_dir}/${file}"
  if [ ! -f "${path}" ]; then
    echo "missing runtime-smoke db-adapter artifact file: ${file}" >&2
    exit 1
  fi
  if [ ! -s "${path}" ]; then
    echo "runtime-smoke db-adapter artifact file is empty: ${file}" >&2
    exit 1
  fi
done

read_field() {
  local file="$1"
  local key="$2"
  sed -n "s/^${key}=//p" "${file}" | head -n 1
}

run_metadata_path="${artifacts_dir}/run-metadata.txt"

source_project="$(read_field "${run_metadata_path}" "sourceProject")"
work_project="$(read_field "${run_metadata_path}" "workProject")"
db_base="$(read_field "${run_metadata_path}" "dbBase")"
db_adapter="$(read_field "${run_metadata_path}" "dbAdapter")"
db_adapter_label="$(read_field "${run_metadata_path}" "dbAdapterLabel")"
port_value="$(read_field "${run_metadata_path}" "port")"
serve_timeout_ms="$(read_field "${run_metadata_path}" "serveTimeoutMs")"
run_flags_value="$(read_field "${run_metadata_path}" "runFlags")"

if [ -z "${source_project}" ]; then
  echo "run-metadata.txt missing sourceProject field" >&2
  exit 1
fi

if [ -z "${work_project}" ]; then
  echo "run-metadata.txt missing workProject field" >&2
  exit 1
fi

if [ "${source_project}" = "${work_project}" ]; then
  echo "run-metadata.txt sourceProject and workProject must differ" >&2
  exit 1
fi

if [ -z "${db_base}" ]; then
  echo "run-metadata.txt missing dbBase field" >&2
  exit 1
fi

if [ "${db_adapter}" != "records-log" ] && [ "${db_adapter}" != "sqlite" ] && [ "${db_adapter}" != "postgres" ]; then
  echo "run-metadata.txt has invalid dbAdapter field" >&2
  exit 1
fi

if [ "${db_adapter_label}" != "records.log" ] && [ "${db_adapter_label}" != "sqlite" ] && [ "${db_adapter_label}" != "postgres" ]; then
  echo "run-metadata.txt has invalid dbAdapterLabel field" >&2
  exit 1
fi

if [ "${db_adapter}" = "records-log" ] && [ "${db_adapter_label}" != "records.log" ]; then
  echo "run-metadata.txt dbAdapterLabel does not match dbAdapter" >&2
  exit 1
fi

if [ "${db_adapter}" = "sqlite" ] && [ "${db_adapter_label}" != "sqlite" ]; then
  echo "run-metadata.txt dbAdapterLabel does not match dbAdapter" >&2
  exit 1
fi

if [ "${db_adapter}" = "postgres" ] && [ "${db_adapter_label}" != "postgres" ]; then
  echo "run-metadata.txt dbAdapterLabel does not match dbAdapter" >&2
  exit 1
fi

if ! [[ "${port_value}" =~ ^[0-9]+$ ]]; then
  echo "run-metadata.txt missing numeric port field" >&2
  exit 1
fi

if [ "${port_value}" -lt 1 ] || [ "${port_value}" -gt 65535 ]; then
  echo "run-metadata.txt port must be between 1 and 65535" >&2
  exit 1
fi

if ! rg -q '^oneshot=true$' "${run_metadata_path}"; then
  echo "run-metadata.txt missing oneshot=true field" >&2
  exit 1
fi

if ! [[ "${serve_timeout_ms}" =~ ^[0-9]+$ ]]; then
  echo "run-metadata.txt missing numeric serveTimeoutMs field" >&2
  exit 1
fi

if [ "${serve_timeout_ms}" -le 0 ]; then
  echo "run-metadata.txt serveTimeoutMs must be greater than 0" >&2
  exit 1
fi

expected_run_flags="--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter"
if [ "${db_adapter}" = "postgres" ]; then
  expected_run_flags="${expected_run_flags},--db-postgres-dsn"
fi
if [ "${run_flags_value}" != "${expected_run_flags}" ]; then
  echo "run-metadata.txt runFlags field does not match expected db-adapter runtime flag shape" >&2
  exit 1
fi

for endpoint in db-exec db-exec-tx db-query-one db-records; do
  headers_path="${artifacts_dir}/${endpoint}.headers"
  run_log_path="${artifacts_dir}/${endpoint}.run.log"

  if ! rg -Fq 'HTTP/1.1 200 OK' "${headers_path}"; then
    echo "${endpoint}.headers missing 200 OK status line" >&2
    exit 1
  fi

  if ! rg -Fq -- '--backend lasm' "${run_log_path}"; then
    echo "${endpoint}.run.log missing --backend lasm invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- "--db-base ${db_base}" "${run_log_path}"; then
    echo "${endpoint}.run.log missing --db-base ${db_base} invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- "--db-adapter ${db_adapter}" "${run_log_path}"; then
    echo "${endpoint}.run.log missing --db-adapter ${db_adapter} invocation token" >&2
    exit 1
  fi

  if [ "${db_adapter}" = "postgres" ] && ! rg -Fq -- "--db-postgres-dsn" "${run_log_path}"; then
    echo "${endpoint}.run.log missing --db-postgres-dsn invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- "--port ${port_value}" "${run_log_path}"; then
    echo "${endpoint}.run.log missing --port ${port_value} invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- '--oneshot' "${run_log_path}"; then
    echo "${endpoint}.run.log missing --oneshot invocation token" >&2
    exit 1
  fi

  if ! rg -Fq -- "--serve-timeout-ms ${serve_timeout_ms}" "${run_log_path}"; then
    echo "${endpoint}.run.log missing --serve-timeout-ms ${serve_timeout_ms} invocation token" >&2
    exit 1
  fi
done

if ! jq -e '.recordId == 1 and .op == "exec" and .db == 1 and .template == "SELECT 1" and .params == "0"' "${artifacts_dir}/db-exec.body" >/dev/null; then
  echo "db-exec.body does not match expected payload contract" >&2
  exit 1
fi

if ! jq -e '.recordId == 2 and .op == "execTx" and (.tx > 0)' "${artifacts_dir}/db-exec-tx.body" >/dev/null; then
  echo "db-exec-tx.body does not match expected payload contract" >&2
  exit 1
fi

if [ "${db_adapter}" = "sqlite" ]; then
  if ! jq -e '.record.op == "queryOne" and .rowSchema == 7 and (.rowObject | type == "object")' "${artifacts_dir}/db-query-one.body" >/dev/null; then
    echo "db-query-one.body does not match expected payload contract" >&2
    exit 1
  fi
else
  if ! jq -e '.record.op == "queryOne" and .rowSchema == 7 and (((.rowObject | type) == "object" and .rowObject.op == "execTx") or ((.row | type) == "string" and (.row | test("\\\"op\\\":\\\"execTx\\\""))))' "${artifacts_dir}/db-query-one.body" >/dev/null; then
    echo "db-query-one.body does not match expected payload contract" >&2
    exit 1
  fi
fi

if ! jq -e --arg adapter "${db_adapter_label}" '.count == 3 and .adapter == $adapter and (.records | type == "array" and length == 3) and (.records[0].op == "exec") and (.records[1].op == "execTx") and (.records[2].op == "queryOne")' "${artifacts_dir}/db-records.body" >/dev/null; then
  echo "db-records.body does not match expected payload contract" >&2
  exit 1
fi

records_log_path="${artifacts_dir}/records.log"
sqlite_path="${artifacts_dir}/records.sqlite3"

if [ "${db_adapter}" = "sqlite" ]; then
  if [ ! -f "${sqlite_path}" ] || [ ! -s "${sqlite_path}" ]; then
    echo "sqlite db-adapter artifacts missing records.sqlite3" >&2
    exit 1
  fi
  if [ -f "${records_log_path}" ]; then
    echo "sqlite db-adapter artifacts should not include records.log" >&2
    exit 1
  fi
elif [ "${db_adapter}" = "records-log" ]; then
  if [ ! -f "${records_log_path}" ] || [ ! -s "${records_log_path}" ]; then
    echo "records-log db-adapter artifacts missing records.log" >&2
    exit 1
  fi
  if [ -f "${sqlite_path}" ]; then
    echo "records-log db-adapter artifacts should not include records.sqlite3" >&2
    exit 1
  fi
  if ! rg -Fq '"op":"exec"' "${records_log_path}" || ! rg -Fq '"op":"execTx"' "${records_log_path}"; then
    echo "records.log does not contain expected deterministic entries" >&2
    exit 1
  fi
  if ! rg -Fq '"op":"queryOne"' "${records_log_path}"; then
    echo "records.log does not contain queryOne persistence entry" >&2
    exit 1
  fi
else
  if [ -f "${records_log_path}" ]; then
    echo "postgres db-adapter artifacts should not include records.log" >&2
    exit 1
  fi
  if [ -f "${sqlite_path}" ]; then
    echo "postgres db-adapter artifacts should not include records.sqlite3" >&2
    exit 1
  fi
fi

echo "runtime-smoke db-adapter artifacts check passed"
