#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="${root_dir}/scripts/check-runtime-smoke-db-adapter-artifacts.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

write_db_adapter_fixture() {
  local root="$1"
  local branch="$2"
  local db_adapter="$3"
  local port="$4"
  local trace_id="$5"
  local serve_timeout_ms="$6"

  local db_adapter_label="records.log"
  case "${db_adapter}" in
    sqlite)
      db_adapter_label="sqlite"
      ;;
    postgres)
      db_adapter_label="postgres"
      ;;
  esac

  local run_flags="--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter"
  if [ "${db_adapter}" = "postgres" ]; then
    run_flags="${run_flags},--db-postgres-dsn"
  fi

  local db_base="/tmp/lasm-db-${branch}"
  local branch_dir="${root}/${branch}"
  mkdir -p "${branch_dir}"

  cat > "${branch_dir}/run-metadata.txt" <<TXT
sourceProject=examples/lasm-alpha-full
workProject=/tmp/lasm-alpha-full-${branch}
dbBase=${db_base}
dbAdapter=${db_adapter}
dbAdapterLabel=${db_adapter_label}
port=${port}
oneshot=true
serveTimeoutMs=${serve_timeout_ms}
runFlags=${run_flags}
TXT

  cat > "${branch_dir}/warmup.log" <<'TXT'
finished dev profile [unoptimized + debuginfo] target(s) in 0.10s
TXT

  cat > "${branch_dir}/db-exec.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  cat > "${branch_dir}/db-exec.body" <<'TXT'
{"recordId":1,"op":"exec","db":1,"template":"SELECT 1","params":"0"}
TXT

  cat > "${branch_dir}/db-exec-tx.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  cat > "${branch_dir}/db-exec-tx.body" <<'TXT'
{"recordId":2,"op":"execTx","db":1,"tx":1,"template":"SELECT 1","params":"0"}
TXT

  cat > "${branch_dir}/db-query-one.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  if [ "${db_adapter}" = "sqlite" ]; then
    cat > "${branch_dir}/db-query-one.body" <<'TXT'
{"ok":true,"record":{"op":"queryOne"},"recordId":3,"row":"{\"1\":1}","rowObject":{"1":1},"rowSchema":7}
TXT
  else
    cat > "${branch_dir}/db-query-one.body" <<'TXT'
{"ok":true,"record":{"op":"queryOne"},"recordId":3,"row":"{\"op\":\"execTx\"}","rowObject":{"op":"execTx"},"rowSchema":7}
TXT
  fi

  cat > "${branch_dir}/db-records.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  cat > "${branch_dir}/db-records.body" <<TXT
{"count":3,"adapter":"${db_adapter_label}","records":[{"recordId":1,"op":"exec"},{"recordId":2,"op":"execTx"},{"recordId":3,"op":"queryOne"}]}
TXT

  local run_cmd="Running \`target/debug/sec4 run --path /tmp/lasm-alpha-full --backend lasm --db-base ${db_base} --db-adapter ${db_adapter} --oneshot --port ${port} --serve-timeout-ms ${serve_timeout_ms}"
  if [ "${db_adapter}" = "postgres" ]; then
    run_cmd="${run_cmd} --db-postgres-dsn postgresql://smoke:smoke@127.0.0.1:5432/smoke\`"
  else
    run_cmd="${run_cmd}\`"
  fi

  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-exec.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-exec-tx.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-query-one.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-records.run.log"

  if [ "${db_adapter}" = "sqlite" ]; then
    printf '%s\n' "sqlite-persisted" > "${branch_dir}/records.sqlite3"
  elif [ "${db_adapter}" = "records-log" ]; then
    cat > "${branch_dir}/records.log" <<'TXT'
{"op":"exec","recordId":1}
{"op":"execTx","recordId":2}
{"op":"queryOne","recordId":3}
TXT
  fi
}

records_ok_dir="${tmp_dir}/ok-records"
write_db_adapter_fixture "${tmp_dir}" "ok-records" "records-log" "8082" "rt-10" "20000"
"${checker}" --artifacts-dir "${records_ok_dir}" >/dev/null

sqlite_ok_dir="${tmp_dir}/ok-sqlite"
write_db_adapter_fixture "${tmp_dir}" "ok-sqlite" "sqlite" "8083" "rt-11" "20000"
"${checker}" --artifacts-dir "${sqlite_ok_dir}" >/dev/null

postgres_ok_dir="${tmp_dir}/ok-postgres"
write_db_adapter_fixture "${tmp_dir}" "ok-postgres" "postgres" "8084" "rt-12" "20000"
"${checker}" --artifacts-dir "${postgres_ok_dir}" >/dev/null

postgres_bad_records_dir="${tmp_dir}/bad-postgres-records-persist"
cp -R "${postgres_ok_dir}" "${postgres_bad_records_dir}"
cat > "${postgres_bad_records_dir}/records.sqlite3" <<'TXT'
unexpected sqlite file
TXT

if "${checker}" --artifacts-dir "${postgres_bad_records_dir}" >"${tmp_dir}/postgres-bad-records.log" 2>&1; then
  echo "expected runtime-smoke db-adapter checker to fail on postgres persistence artifacts" >&2
  exit 1
fi

if ! rg -Fq 'postgres db-adapter artifacts should not include records.sqlite3' "${tmp_dir}/postgres-bad-records.log"; then
  echo "expected postgres persistence diagnostic for unexpected records.sqlite3" >&2
  exit 1
fi

missing_sqlite_dir="${tmp_dir}/bad-missing-sqlite"
cp -R "${sqlite_ok_dir}" "${missing_sqlite_dir}"
rm -f "${missing_sqlite_dir}/records.sqlite3"

if "${checker}" --artifacts-dir "${missing_sqlite_dir}" >"${tmp_dir}/missing-sqlite.log" 2>&1; then
  echo "expected runtime-smoke db-adapter checker to fail on missing sqlite persistence file" >&2
  exit 1
fi

if ! rg -Fq 'sqlite db-adapter artifacts missing records.sqlite3' "${tmp_dir}/missing-sqlite.log"; then
  echo "expected sqlite persistence diagnostic for missing records.sqlite3" >&2
  exit 1
fi

runflags_bad_dir="${tmp_dir}/bad-runflags"
cp -R "${records_ok_dir}" "${runflags_bad_dir}"
cat > "${runflags_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/lasm-alpha-full
workProject=/tmp/lasm-alpha-full-ok-records
dbBase=/tmp/lasm-db-ok-records
dbAdapter=records-log
dbAdapterLabel=records.log
port=8082
oneshot=true
serveTimeoutMs=20000
runFlags=--port,--oneshot,--serve-timeout-ms,--db-base
TXT

if "${checker}" --artifacts-dir "${runflags_bad_dir}" >"${tmp_dir}/runflags-bad.log" 2>&1; then
  echo "expected runtime-smoke db-adapter checker to fail on runFlags shape drift" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt runFlags field does not match expected db-adapter runtime flag shape' "${tmp_dir}/runflags-bad.log"; then
  echo "expected runFlags-shape diagnostic for db-adapter metadata" >&2
  exit 1
fi

label_bad_dir="${tmp_dir}/bad-label"
cp -R "${records_ok_dir}" "${label_bad_dir}"
cat > "${label_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/lasm-alpha-full
workProject=/tmp/lasm-alpha-full-ok-records
dbBase=/tmp/lasm-db-ok-records
dbAdapter=records-log
dbAdapterLabel=sqlite
port=8082
oneshot=true
serveTimeoutMs=20000
runFlags=--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter
TXT

if "${checker}" --artifacts-dir "${label_bad_dir}" >"${tmp_dir}/label-bad.log" 2>&1; then
  echo "expected runtime-smoke db-adapter checker to fail when dbAdapterLabel mismatches dbAdapter" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt dbAdapterLabel does not match dbAdapter' "${tmp_dir}/label-bad.log"; then
  echo "expected adapter-label mismatch diagnostic" >&2
  exit 1
fi

payload_bad_dir="${tmp_dir}/bad-records-payload"
cp -R "${sqlite_ok_dir}" "${payload_bad_dir}"
cat > "${payload_bad_dir}/db-records.body" <<'TXT'
{"count":3,"adapter":"records.log","records":[{"recordId":1,"op":"exec"},{"recordId":2,"op":"execTx"},{"recordId":3,"op":"queryOne"}]}
TXT

if "${checker}" --artifacts-dir "${payload_bad_dir}" >"${tmp_dir}/payload-bad.log" 2>&1; then
  echo "expected runtime-smoke db-adapter checker to fail on db-records payload adapter mismatch" >&2
  exit 1
fi

if ! rg -Fq 'db-records.body does not match expected payload contract' "${tmp_dir}/payload-bad.log"; then
  echo "expected db-records payload diagnostic" >&2
  exit 1
fi

echo "runtime-smoke db-adapter artifacts checker test passed"
