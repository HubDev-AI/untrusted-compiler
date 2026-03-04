#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--project <path>] [--db-adapter <records-log|sqlite|postgres>] [--db-base <path>] [--db-postgres-dsn <dsn>] [--db-postgres-dsn-file <path>] [--artifacts-dir <path>] [--serve-timeout-ms <ms>]

Builds and runs a temporary lasm-alpha-full service via `sec4 run` in oneshot mode,
executes deterministic DB intrinsic requests, and validates adapter-specific persistence.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_project="${root_dir}/examples/lasm-alpha-full"
db_adapter="records-log"
db_base=""
db_postgres_dsn=""
db_postgres_dsn_file=""
artifacts_dir=""
serve_timeout_ms="20000"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --project)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      source_project="$2"
      shift 2
      ;;
    --project=*)
      source_project="${1#--project=}"
      shift
      ;;
    --db-adapter)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_adapter="$2"
      shift 2
      ;;
    --db-adapter=*)
      db_adapter="${1#--db-adapter=}"
      shift
      ;;
    --db-base)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_base="$2"
      shift 2
      ;;
    --db-base=*)
      db_base="${1#--db-base=}"
      shift
      ;;
    --db-postgres-dsn)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_postgres_dsn="$2"
      shift 2
      ;;
    --db-postgres-dsn=*)
      db_postgres_dsn="${1#--db-postgres-dsn=}"
      shift
      ;;
    --db-postgres-dsn-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      db_postgres_dsn_file="$2"
      shift 2
      ;;
    --db-postgres-dsn-file=*)
      db_postgres_dsn_file="${1#--db-postgres-dsn-file=}"
      shift
      ;;
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
    --serve-timeout-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      serve_timeout_ms="$2"
      shift 2
      ;;
    --serve-timeout-ms=*)
      serve_timeout_ms="${1#--serve-timeout-ms=}"
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

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "missing required command: ${name}" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd curl
require_cmd jq
require_cmd python3

if [ ! -d "${source_project}" ]; then
  echo "missing project directory: ${source_project}" >&2
  exit 1
fi

if ! [[ "${serve_timeout_ms}" =~ ^[0-9]+$ ]] || [ "${serve_timeout_ms}" -le 0 ]; then
  echo "invalid --serve-timeout-ms value (expected positive integer): ${serve_timeout_ms}" >&2
  exit 1
fi

case "${db_adapter}" in
  records-log|sqlite|postgres)
    ;;
  *)
    echo "invalid --db-adapter value (expected records-log, sqlite or postgres): ${db_adapter}" >&2
    exit 1
    ;;
esac

if [ "${db_adapter}" != "postgres" ] && [ -n "${db_postgres_dsn}" ]; then
  echo "--db-postgres-dsn requires --db-adapter postgres" >&2
  exit 1
fi

if [ "${db_adapter}" != "postgres" ] && [ -n "${db_postgres_dsn_file}" ]; then
  echo "--db-postgres-dsn-file requires --db-adapter postgres" >&2
  exit 1
fi

resolve_env_value() {
  local key="$1"
  if [ -n "${!key+x}" ]; then
    printf '%s' "${!key}"
  fi
}

strip_quotes() {
  local raw="$1"
  local trimmed
  trimmed="$(printf '%s' "${raw}" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"

  if [ "${#trimmed}" -ge 2 ]; then
    case "${trimmed}" in
      \"*\" )
        trimmed="${trimmed#\"}"
        trimmed="${trimmed%\"}"
        ;;
      \'*\' )
        trimmed="${trimmed#\'}"
        trimmed="${trimmed%\'}"
        ;;
    esac
  fi

  printf '%s' "${trimmed}"
}

resolve_postgres_dsn_file() {
  local dsn_file="$1"
  if [ ! -f "${dsn_file}" ]; then
    echo "" 
    return 1
  fi

  while IFS= read -r line; do
    line="${line%$'\n'}"
    line="$(printf '%s' "$line" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    if [ -z "${line}" ] || [[ "${line}" == \#* ]]; then
      continue
    fi

    if [[ "${line}" == export* ]]; then
      line="${line#export }"
    fi

    if [[ "${line}" == *"="* ]]; then
      local key="${line%%=*}"
      if [ "${key}" != "SEC4_DB_ALPHA_DB_POSTGRES_DSN" ] && [ "${key}" != "SEC4_RT_LASM_DB_POSTGRES_DSN" ] && [ "${key}" != "SEC4_DB_ALPHA_POSTGRES_DSN_FILE" ] && [ "${key}" != "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE" ] && [ "${key}" != "SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH" ] && [ "${key}" != "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH" ]; then
        continue
      fi
      local value="${line#*=}"
      value="$(strip_quotes "${value}")"
      if [ -n "${value}" ]; then
        printf '%s' "${value}"
        return 0
      fi
    else
      printf '%s' "${line}"
      return 0
    fi
  done < "${dsn_file}"

  echo ""
}

resolve_dsn_from_file_candidates() {
  local candidate_one
  local candidate_two
  local value
  candidate_one="$(resolve_env_value SEC4_DB_ALPHA_DB_POSTGRES_DSN_FILE)"
  candidate_two="$(resolve_env_value SEC4_RT_LASM_DB_POSTGRES_DSN_FILE)"
  if [ -n "${candidate_one}" ] && [ -n "${candidate_two}" ] && [ "${candidate_one}" != "${candidate_two}" ]; then
    echo "db adapter postgres environment DSN file source is ambiguous: SEC4_DB_ALPHA_DB_POSTGRES_DSN_FILE and SEC4_RT_LASM_DB_POSTGRES_DSN_FILE"
    return 1
  fi

  local candidate_three
  local candidate_four
  candidate_three="$(resolve_env_value SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH)"
  candidate_four="$(resolve_env_value SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH)"
  if [ -n "${candidate_three}" ] && [ -n "${candidate_four}" ] && [ "${candidate_three}" != "${candidate_four}" ]; then
    echo "db adapter postgres environment DSN file source is ambiguous: SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH and SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH"
    return 1
  fi

  for dsn_file in "${db_postgres_dsn_file}" "${candidate_one}" "${candidate_two}" "${candidate_three}" "${candidate_four}"; do
    if [ -z "${dsn_file}" ]; then
      continue
    fi
    if [ ! -f "${dsn_file}" ]; then
      continue
    fi
    value="$(resolve_postgres_dsn_file "${dsn_file}")"
    if [ -n "${value}" ]; then
      printf '%s' "${value}"
      return 0
    fi
  done

  echo ""
}

resolve_postgres_dsn() {
  local resolved=""

  if [ -n "${db_postgres_dsn}" ]; then
    if [ -z "${db_postgres_dsn}" ]; then
      echo "empty --db-postgres-dsn value" >&2
      exit 1
    fi
    printf '%s' "${db_postgres_dsn}"
    return 0
  fi

  if [ -n "${db_postgres_dsn_file}" ]; then
    if [ ! -f "${db_postgres_dsn_file}" ]; then
      echo "db-postgres-dsn-file does not exist: ${db_postgres_dsn_file}" >&2
      exit 1
    fi
    resolved="$(resolve_postgres_dsn_file "${db_postgres_dsn_file}")"
    if [ -z "${resolved}" ]; then
      echo "db-postgres-dsn-file is empty: ${db_postgres_dsn_file}" >&2
      exit 1
    fi
    printf '%s' "${resolved}"
    return 0
  fi

  resolved="$(resolve_env_value SEC4_DB_ALPHA_DB_POSTGRES_DSN)"
  if [ -n "${resolved}" ]; then
    printf '%s' "${resolved}"
    return 0
  fi

  resolved="$(resolve_env_value SEC4_RT_LASM_DB_POSTGRES_DSN)"
  if [ -n "${resolved}" ]; then
    printf '%s' "${resolved}"
    return 0
  fi

  resolved="$(resolve_dsn_from_file_candidates)"
  if [ -n "${resolved}" ]; then
    printf '%s' "${resolved}"
    return 0
  fi

  return 1
}

if [ "${db_adapter}" = "postgres" ]; then
  db_postgres_dsn="$(resolve_postgres_dsn || true)"
  if [ -z "${db_postgres_dsn}" ]; then
    echo "postgres adapter requires --db-postgres-dsn or DSN from file/env sources" >&2
    exit 1
  fi
fi

pick_free_port() {
  python3 - <<'PY'
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

tmp_dir="$(mktemp -d)"
cleanup() {
  if [ -n "${service_pid:-}" ] && kill -0 "${service_pid}" >/dev/null 2>&1; then
    kill "${service_pid}" >/dev/null 2>&1 || true
    wait "${service_pid}" >/dev/null 2>&1 || true
  fi
  if [ -n "${artifacts_dir}" ]; then
    mkdir -p "${artifacts_dir}"
    for artifact in \
      run-metadata.txt warmup.log \
      db-exec.headers db-exec.body db-exec.run.log \
      db-exec-tx.headers db-exec-tx.body db-exec-tx.run.log \
      db-query-one.headers db-query-one.body db-query-one.run.log \
      db-records.headers db-records.body db-records.run.log; do
      if [ -f "${tmp_dir}/${artifact}" ]; then
        cp "${tmp_dir}/${artifact}" "${artifacts_dir}/${artifact}"
      fi
    done
    if [ -f "${db_base}/records.log" ]; then
      cp "${db_base}/records.log" "${artifacts_dir}/records.log"
    fi
    if [ -f "${db_base}/records.sqlite3" ]; then
      cp "${db_base}/records.sqlite3" "${artifacts_dir}/records.sqlite3"
    fi
  fi
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

work_project="${tmp_dir}/lasm-alpha-full"
cp -R "${source_project}" "${work_project}"

if [ -z "${db_base}" ]; then
  db_base="${tmp_dir}/lasm-db"
fi
mkdir -p "${db_base}"

port="$(pick_free_port)"

case "${db_adapter}" in
  records-log)
    adapter_label="records.log"
    ;;
  sqlite)
    adapter_label="sqlite"
    ;;
  postgres)
    adapter_label="postgres"
    ;;
esac

run_flags="--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter"
if [ "${db_adapter}" = "postgres" ]; then
  run_flags="${run_flags},--db-postgres-dsn"
fi

cat > "${tmp_dir}/run-metadata.txt" <<META
sourceProject=${source_project}
workProject=${work_project}
dbBase=${db_base}
dbAdapter=${db_adapter}
dbAdapterLabel=${adapter_label}
port=${port}
oneshot=true
serveTimeoutMs=${serve_timeout_ms}
runFlags=${run_flags}
META

warmup_log="${tmp_dir}/warmup.log"
if ! (cd "${root_dir}" && cargo build -p sec4 >"${warmup_log}" 2>&1); then
  echo "sec4 warmup build failed before LASM DB adapter smoke requests" >&2
  cat "${warmup_log}" >&2
  exit 1
fi

request_once() {
  local name="$1"
  local method="$2"
  local path="$3"
  local expected_status="$4"
  local headers_file="${tmp_dir}/${name}.headers"
  local body_file="${tmp_dir}/${name}.body"
  local log_file="${tmp_dir}/${name}.run.log"
  shift 4

  local -a curl_args=(
    --silent --show-error
    --output "${body_file}"
    --dump-header "${headers_file}"
    --write-out "%{http_code}"
    --request "${method}"
    --header "Authorization: Bearer smoke-token"
    "$@"
    "http://127.0.0.1:${port}${path}"
  )

  local -a run_cmd=(
    cargo run -p sec4 -- run
    --path "${work_project}"
    --backend lasm
    --db-base "${db_base}"
    --db-adapter "${db_adapter}"
    --oneshot
    --port "${port}"
    --serve-timeout-ms "${serve_timeout_ms}"
  )

  if [ "${db_adapter}" = "postgres" ]; then
    run_cmd+=(--db-postgres-dsn "${db_postgres_dsn}")
  fi

  "${run_cmd[@]}" >"${log_file}" 2>&1 &
  service_pid=$!

  local status_code=""
  local got_response=0
  local response_wait_ms=$((serve_timeout_ms + 45000))
  local response_poll_ms=50
  local response_max_attempts=$(((response_wait_ms + response_poll_ms - 1) / response_poll_ms))
  local attempt=0
  while [ "${attempt}" -lt "${response_max_attempts}" ]; do
    if ! kill -0 "${service_pid}" >/dev/null 2>&1; then
      echo "sec4 run exited before ${name} request completed" >&2
      cat "${log_file}" >&2
      exit 1
    fi

    status_code="$(curl "${curl_args[@]}" 2>/dev/null || true)"
    if [ -n "${status_code}" ] && [ "${status_code}" != "000" ]; then
      got_response=1
      break
    fi
    sleep 0.05
    attempt=$((attempt + 1))
  done

  local wait_status=""
  local wait_attempt=0
  while [ "${wait_attempt}" -lt 600 ]; do
    if wait "${service_pid}" >/dev/null 2>&1; then
      wait_status="ok"
      break
    fi
    sleep 0.02
    wait_attempt=$((wait_attempt + 1))
  done
  service_pid=""

  if [ -z "${wait_status}" ]; then
    echo "sec4 run did not exit after oneshot request for ${name}" >&2
    cat "${log_file}" >&2
    exit 1
  fi

  if [ "${status_code}" != "${expected_status}" ]; then
    echo "unexpected HTTP status for ${name}: got ${status_code}, expected ${expected_status}" >&2
    cat "${headers_file}" >&2
    cat "${body_file}" >&2
    exit 1
  fi

  echo "${body_file}"
}

exec_body="$(request_once "db-exec" "POST" "/db/exec?template=SELECT%201&params=alpha" "200")"
if ! jq -e '.recordId == 1 and .op == "exec" and .db == 1 and .template == "SELECT 1" and .params == "alpha"' "${exec_body}" >/dev/null; then
  echo "unexpected /db/exec response payload" >&2
  cat "${exec_body}" >&2
  exit 1
fi

exec_tx_body="$(request_once "db-exec-tx" "POST" "/db/exec-tx?template=SELECT%201&params=alpha" "200")"
if ! jq -e '.recordId == 2 and .op == "execTx" and (.tx > 0)' "${exec_tx_body}" >/dev/null; then
  echo "unexpected /db/exec-tx response payload" >&2
  cat "${exec_tx_body}" >&2
  exit 1
fi

query_one_body="$(request_once "db-query-one" "GET" "/db/query-one?template=SELECT%201&params=alpha&row_schema=7" "200")"
if ! jq -e '.record.op == "queryOne" and .rowSchema == 7 and ((.rowObject | type == "object" and .op == "execTx") or (.row | type == "string" and (.row|test("\"op\":\"execTx\""))))' "${query_one_body}" >/dev/null; then
  echo "unexpected /db/query-one response payload" >&2
  cat "${query_one_body}" >&2
  exit 1
fi

records_body="$(request_once "db-records" "GET" "/db/records?includeRecords=true" "200")"
if ! jq -e --arg adapter "${adapter_label}" '.count >= 3 and .adapter == $adapter and (.records | length >= 3) and (.records[0].op == "exec") and (.records[1].op == "execTx") and (.records[2].op == "queryOne")' "${records_body}" >/dev/null; then
  echo "unexpected /db/records response payload" >&2
  cat "${records_body}" >&2
  exit 1
fi

records_log_path="${db_base}/records.log"
sqlite_path="${db_base}/records.sqlite3"
if [ "${db_adapter}" = "sqlite" ]; then
  if [ ! -f "${sqlite_path}" ]; then
    echo "sqlite adapter smoke expected persisted records.sqlite3" >&2
    exit 1
  fi
  if [ -f "${records_log_path}" ]; then
    echo "sqlite adapter smoke should not emit records.log" >&2
    exit 1
  fi
elif [ "${db_adapter}" = "records-log" ]; then
  if [ ! -f "${records_log_path}" ]; then
    echo "records-log adapter smoke expected persisted records.log" >&2
    exit 1
  fi
  if [ -f "${sqlite_path}" ]; then
    echo "records-log adapter smoke should not emit records.sqlite3" >&2
    exit 1
  fi
  if ! rg -Fq '"op":"exec"' "${records_log_path}" || ! rg -Fq '"op":"execTx"' "${records_log_path}"; then
    echo "records.log should contain deterministic exec/execTx entries" >&2
    cat "${records_log_path}" >&2
    exit 1
  fi
else
  if [ -f "${records_log_path}" ] || [ -f "${sqlite_path}" ]; then
    echo "postgres adapter smoke should not persist records.log/records.sqlite3" >&2
    exit 1
  fi
fi

echo "sec4 run lasm db-adapter smoke passed"
