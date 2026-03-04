#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bundle_checker="${root_dir}/scripts/check-runtime-smoke-bundle.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

write_branch_fixture() {
  local root="$1"
  local branch="$2"
  local max_body="$3"
  local run_flags="$4"
  local trace_id="$5"
  local port="$6"
  local include_max_body_token="$7"

  local branch_dir="${root}/${branch}"
  mkdir -p "${branch_dir}"

  cat > "${branch_dir}/run-metadata.txt" <<TXT
sourceProject=examples/hello-api
workProject=/tmp/hello-api-${branch}
port=${port}
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=${max_body}
runFlags=${run_flags}
TXT

  cat > "${branch_dir}/health.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
X-Trace-Id: ${trace_id}
TXT

  cat > "${branch_dir}/health.body" <<'TXT'
ok
TXT

  cat > "${branch_dir}/users.headers" <<TXT
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT

  cat > "${branch_dir}/users.body" <<TXT
{"ok":true,"status":201,"traceId":"${trace_id}","timeMs":1,"data":1}
TXT

  local run_cmd="Running \`target/debug/sec4 run --path /tmp/hello-api --port ${port} --oneshot --serve-timeout-ms 12000"
  if [ "${include_max_body_token}" = "true" ]; then
    run_cmd="${run_cmd} --max-body-bytes ${max_body}"
  fi
  run_cmd="${run_cmd}\`"

  printf '%s\n' "${run_cmd}" > "${branch_dir}/health.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/users.run.log"
}

write_db_adapter_branch_fixture() {
  local root="$1"
  local branch="$2"
  local db_adapter="$3"
  local port="$4"
  local trace_id="$5"
  local serve_timeout_ms="$6"

  local db_adapter_label="records.log"
  if [ "${db_adapter}" = "sqlite" ]; then
    db_adapter_label="sqlite"
  elif [ "${db_adapter}" = "postgres" ]; then
    db_adapter_label="postgres"
  fi

  local db_base="/tmp/lasm-db-${branch}"
  local run_flags="--port,--oneshot,--serve-timeout-ms,--db-base,--db-adapter"
  if [ "${db_adapter}" = "postgres" ]; then
    run_flags="${run_flags},--db-postgres-dsn"
  fi
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
{"recordId":1,"op":"exec","db":1,"template":"SELECT 1","params":"alpha"}
TXT

  cat > "${branch_dir}/db-exec-tx.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  cat > "${branch_dir}/db-exec-tx.body" <<'TXT'
{"recordId":2,"op":"execTx","db":1,"tx":1,"template":"SELECT 1","params":"alpha"}
TXT

  cat > "${branch_dir}/db-query-one.headers" <<TXT
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT
  cat > "${branch_dir}/db-query-one.body" <<'TXT'
{"recordId":2,"rowSchema":7,"summary":"op=execTx"}
TXT

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
    run_cmd="${run_cmd} --db-postgres-dsn postgresql://smoke:smoke@127.0.0.1:5432/smoke"
  fi
  run_cmd="${run_cmd}\`"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-exec.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-exec-tx.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-query-one.run.log"
  printf '%s\n' "${run_cmd}" > "${branch_dir}/db-records.run.log"

  if [ "${db_adapter}" = "records-log" ]; then
    cat > "${branch_dir}/records.log" <<'TXT'
{"op":"exec","recordId":1}
{"op":"execTx","recordId":2}
{"op":"queryOne","recordId":3}
TXT
  elif [ "${db_adapter}" = "sqlite" ]; then
    printf '%s\n' "sqlite-persisted" > "${branch_dir}/records.sqlite3"
  fi
}

ok_root="${tmp_dir}/ok"
write_branch_fixture "${ok_root}" "default" "unset" "--port,--oneshot,--serve-timeout-ms" "rt-1" "8080" "false"
write_branch_fixture "${ok_root}" "max-body" "2048" "--port,--oneshot,--serve-timeout-ms,--max-body-bytes" "rt-2" "8081" "true"
write_db_adapter_branch_fixture "${ok_root}" "lasm-db-records-log" "records-log" "8082" "rt-10" "20000"
write_db_adapter_branch_fixture "${ok_root}" "lasm-db-sqlite" "sqlite" "8083" "rt-11" "20000"
write_db_adapter_branch_fixture "${ok_root}" "lasm-db-postgres" "postgres" "8084" "rt-12" "20000"

ok_index="${ok_root}/runtime-smoke-branch-index.json"
LASM_SMOKE_REQUIRE_POSTGRES_DB_ADAPTER=1 "${bundle_checker}" --artifacts-root "${ok_root}" --index-path "${ok_index}" >/dev/null

if ! jq -e '.branchOrder == ["default","max-body"] and (.branches | length == 2)' "${ok_index}" >/dev/null; then
  echo "expected runtime-smoke bundle checker to generate valid branch index" >&2
  exit 1
fi

missing_branch_root="${tmp_dir}/bad-missing-branch"
cp -R "${ok_root}" "${missing_branch_root}"
rm -rf "${missing_branch_root}/max-body"

if "${bundle_checker}" --artifacts-root "${missing_branch_root}" --index-path "${missing_branch_root}/runtime-smoke-branch-index.json" >"${tmp_dir}/missing-branch.log" 2>&1; then
  echo "expected runtime-smoke bundle checker to fail on missing max-body branch" >&2
  exit 1
fi

if ! rg -Fq 'runtime-smoke bundle missing branch directory: max-body' "${tmp_dir}/missing-branch.log"; then
  echo "expected missing-branch diagnostic for bundle checker" >&2
  exit 1
fi

missing_db_branch_root="${tmp_dir}/bad-missing-db-branch"
cp -R "${ok_root}" "${missing_db_branch_root}"
rm -rf "${missing_db_branch_root}/lasm-db-sqlite"

if "${bundle_checker}" --artifacts-root "${missing_db_branch_root}" --index-path "${missing_db_branch_root}/runtime-smoke-branch-index.json" >"${tmp_dir}/missing-db-branch.log" 2>&1; then
  echo "expected runtime-smoke bundle checker to fail on missing lasm-db-sqlite branch" >&2
  exit 1
fi

if ! rg -Fq 'runtime-smoke bundle missing branch directory: lasm-db-sqlite' "${tmp_dir}/missing-db-branch.log"; then
  echo "expected missing-branch diagnostic for sqlite db-adapter branch" >&2
  exit 1
fi

missing_postgres_db_branch_root="${tmp_dir}/bad-missing-postgres-db-branch"
cp -R "${ok_root}" "${missing_postgres_db_branch_root}"
rm -rf "${missing_postgres_db_branch_root}/lasm-db-postgres"

if LASM_SMOKE_REQUIRE_POSTGRES_DB_ADAPTER=1 "${bundle_checker}" --artifacts-root "${missing_postgres_db_branch_root}" --index-path "${missing_postgres_db_branch_root}/runtime-smoke-branch-index.json" >"${tmp_dir}/missing-postgres-db-branch.log" 2>&1; then
  echo "expected runtime-smoke bundle checker to fail on missing lasm-db-postgres branch" >&2
  exit 1
fi

if ! rg -Fq 'runtime-smoke bundle missing branch directory: lasm-db-postgres' "${tmp_dir}/missing-postgres-db-branch.log"; then
  echo "expected missing-branch diagnostic for postgres db-adapter branch" >&2
  exit 1
fi

shape_bad_root="${tmp_dir}/bad-shape"
cp -R "${ok_root}" "${shape_bad_root}"
cat > "${shape_bad_root}/max-body/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api-max-body
port=8081
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=2048
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${bundle_checker}" --artifacts-root "${shape_bad_root}" --index-path "${shape_bad_root}/runtime-smoke-branch-index.json" >"${tmp_dir}/shape-bad.log" 2>&1; then
  echo "expected runtime-smoke bundle checker to fail on max-body runFlags shape drift" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt runFlags field does not match expected runtime flag shape' "${tmp_dir}/shape-bad.log"; then
  echo "expected runFlags-shape diagnostic to bubble through bundle checker" >&2
  exit 1
fi

db_shape_bad_root="${tmp_dir}/bad-db-shape"
cp -R "${ok_root}" "${db_shape_bad_root}"
cat > "${db_shape_bad_root}/lasm-db-sqlite/run-metadata.txt" <<'TXT'
sourceProject=examples/lasm-alpha-full
workProject=/tmp/lasm-alpha-full-lasm-db-sqlite
dbBase=/tmp/lasm-db-lasm-db-sqlite
dbAdapter=sqlite
dbAdapterLabel=sqlite
port=8083
oneshot=true
serveTimeoutMs=20000
runFlags=--port,--oneshot,--serve-timeout-ms,--db-base
TXT

if "${bundle_checker}" --artifacts-root "${db_shape_bad_root}" --index-path "${db_shape_bad_root}/runtime-smoke-branch-index.json" >"${tmp_dir}/db-shape-bad.log" 2>&1; then
  echo "expected runtime-smoke bundle checker to fail on db-adapter runFlags shape drift" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt runFlags field does not match expected db-adapter runtime flag shape' "${tmp_dir}/db-shape-bad.log"; then
  echo "expected db-adapter runFlags-shape diagnostic to bubble through bundle checker" >&2
  exit 1
fi

echo "runtime-smoke bundle checker test passed"
