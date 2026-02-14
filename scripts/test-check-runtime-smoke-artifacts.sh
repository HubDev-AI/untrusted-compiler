#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="${root_dir}/scripts/check-runtime-smoke-artifacts.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

ok_dir="${tmp_dir}/ok"
mkdir -p "${ok_dir}"

cat > "${ok_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

cat > "${ok_dir}/health.headers" <<'TXT'
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
X-Trace-Id: rt-1
TXT

cat > "${ok_dir}/health.body" <<'TXT'
ok
TXT

cat > "${ok_dir}/health.run.log" <<'TXT'
Running `target/debug/sec4 run --path /tmp/hello-api --port 8080 --oneshot --serve-timeout-ms 12000`
TXT

cat > "${ok_dir}/users.headers" <<'TXT'
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
X-Trace-Id: rt-1
TXT

cat > "${ok_dir}/users.body" <<'TXT'
{"ok":true,"status":201,"traceId":"rt-1","timeMs":1,"data":1}
TXT

cat > "${ok_dir}/users.run.log" <<'TXT'
Running `target/debug/sec4 run --path /tmp/hello-api --port 8080 --oneshot --serve-timeout-ms 12000`
TXT

"${checker}" --artifacts-dir "${ok_dir}" >/dev/null

bad_dir="${tmp_dir}/bad-missing"
cp -R "${ok_dir}" "${bad_dir}"
rm -f "${bad_dir}/users.body"

if "${checker}" --artifacts-dir "${bad_dir}" >"${tmp_dir}/bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing users.body" >&2
  exit 1
fi

if ! rg -Fq 'missing runtime-smoke artifact file: users.body' "${tmp_dir}/bad.log"; then
  echo "expected missing-artifact diagnostic for users.body" >&2
  exit 1
fi

health_trace_missing_dir="${tmp_dir}/bad-health-trace-missing"
cp -R "${ok_dir}" "${health_trace_missing_dir}"
cat > "${health_trace_missing_dir}/health.headers" <<'TXT'
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
TXT

if "${checker}" --artifacts-dir "${health_trace_missing_dir}" >"${tmp_dir}/health-trace-missing.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing health trace header" >&2
  exit 1
fi

if ! rg -Fq 'health.headers missing X-Trace-Id header' "${tmp_dir}/health-trace-missing.log"; then
  echo "expected missing-header diagnostic for health trace id" >&2
  exit 1
fi

health_trace_bad_dir="${tmp_dir}/bad-health-trace-format"
cp -R "${ok_dir}" "${health_trace_bad_dir}"
cat > "${health_trace_bad_dir}/health.headers" <<'TXT'
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
X-Trace-Id: trace-1
TXT

if "${checker}" --artifacts-dir "${health_trace_bad_dir}" >"${tmp_dir}/health-trace-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on malformed health trace header" >&2
  exit 1
fi

if ! rg -Fq 'health.headers contains malformed X-Trace-Id value' "${tmp_dir}/health-trace-bad.log"; then
  echo "expected malformed-header diagnostic for health trace id" >&2
  exit 1
fi

envelope_bad_dir="${tmp_dir}/bad-envelope"
cp -R "${ok_dir}" "${envelope_bad_dir}"
cat > "${envelope_bad_dir}/users.body" <<'TXT'
{"ok":true,"status":201,"traceId":"rt-1","data":1}
TXT

if "${checker}" --artifacts-dir "${envelope_bad_dir}" >"${tmp_dir}/envelope-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on malformed success envelope" >&2
  exit 1
fi

if ! rg -Fq 'users.body does not match expected std-success envelope contract' "${tmp_dir}/envelope-bad.log"; then
  echo "expected envelope-contract diagnostic for malformed users.body" >&2
  exit 1
fi

trace_bad_dir="${tmp_dir}/bad-trace"
cp -R "${ok_dir}" "${trace_bad_dir}"
cat > "${trace_bad_dir}/users.body" <<'TXT'
{"ok":true,"status":201,"traceId":"trace-1","timeMs":1,"data":1}
TXT

if "${checker}" --artifacts-dir "${trace_bad_dir}" >"${tmp_dir}/trace-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on malformed traceId" >&2
  exit 1
fi

if ! rg -Fq 'users.body does not match expected std-success envelope contract' "${tmp_dir}/trace-bad.log"; then
  echo "expected envelope-contract diagnostic for malformed traceId" >&2
  exit 1
fi

trace_header_missing_dir="${tmp_dir}/bad-trace-header-missing"
cp -R "${ok_dir}" "${trace_header_missing_dir}"
cat > "${trace_header_missing_dir}/users.headers" <<'TXT'
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
TXT

if "${checker}" --artifacts-dir "${trace_header_missing_dir}" >"${tmp_dir}/trace-header-missing.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing users trace header" >&2
  exit 1
fi

if ! rg -Fq 'users.headers missing X-Trace-Id header' "${tmp_dir}/trace-header-missing.log"; then
  echo "expected missing-header diagnostic for users trace id" >&2
  exit 1
fi

trace_mismatch_dir="${tmp_dir}/bad-trace-mismatch"
cp -R "${ok_dir}" "${trace_mismatch_dir}"
cat > "${trace_mismatch_dir}/users.headers" <<'TXT'
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
X-Trace-Id: rt-999
TXT

if "${checker}" --artifacts-dir "${trace_mismatch_dir}" >"${tmp_dir}/trace-mismatch.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on header/body trace mismatch" >&2
  exit 1
fi

if ! rg -Fq 'users traceId mismatch between headers and body' "${tmp_dir}/trace-mismatch.log"; then
  echo "expected trace-mismatch diagnostic for users header/body trace ids" >&2
  exit 1
fi

health_log_bad_dir="${tmp_dir}/bad-health-log"
cp -R "${ok_dir}" "${health_log_bad_dir}"
cat > "${health_log_bad_dir}/health.run.log" <<'TXT'
Running `target/debug/sec4 run --path /tmp/hello-api --port 8080 --serve-timeout-ms 12000`
TXT

if "${checker}" --artifacts-dir "${health_log_bad_dir}" >"${tmp_dir}/health-log-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing health --oneshot token" >&2
  exit 1
fi

if ! rg -Fq 'health.run.log missing --oneshot invocation token' "${tmp_dir}/health-log-bad.log"; then
  echo "expected missing-token diagnostic for health run log --oneshot" >&2
  exit 1
fi

users_log_bad_dir="${tmp_dir}/bad-users-log"
cp -R "${ok_dir}" "${users_log_bad_dir}"
cat > "${users_log_bad_dir}/users.run.log" <<'TXT'
Running `target/debug/sec4 run --path /tmp/hello-api --port 8080 --oneshot`
TXT

if "${checker}" --artifacts-dir "${users_log_bad_dir}" >"${tmp_dir}/users-log-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing users --serve-timeout-ms token" >&2
  exit 1
fi

if ! rg -Fq 'users.run.log missing --serve-timeout-ms 12000 invocation token' "${tmp_dir}/users-log-bad.log"; then
  echo "expected missing-token diagnostic for users run log --serve-timeout-ms" >&2
  exit 1
fi

port_mismatch_dir="${tmp_dir}/bad-log-port-mismatch"
cp -R "${ok_dir}" "${port_mismatch_dir}"
cat > "${port_mismatch_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=9090
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${port_mismatch_dir}" >"${tmp_dir}/port-mismatch.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail when run-log port and metadata port diverge" >&2
  exit 1
fi

if ! rg -Fq 'health.run.log missing --port 9090 invocation token' "${tmp_dir}/port-mismatch.log"; then
  echo "expected missing-token diagnostic for health run log metadata-correlated port" >&2
  exit 1
fi

timeout_mismatch_dir="${tmp_dir}/bad-log-timeout-mismatch"
cp -R "${ok_dir}" "${timeout_mismatch_dir}"
cat > "${timeout_mismatch_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=9000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${timeout_mismatch_dir}" >"${tmp_dir}/timeout-mismatch.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail when run-log timeout and metadata timeout diverge" >&2
  exit 1
fi

if ! rg -Fq 'health.run.log missing --serve-timeout-ms 9000 invocation token' "${tmp_dir}/timeout-mismatch.log"; then
  echo "expected missing-token diagnostic for health run log metadata-correlated timeout" >&2
  exit 1
fi

port_range_bad_dir="${tmp_dir}/bad-port-range"
cp -R "${ok_dir}" "${port_range_bad_dir}"
cat > "${port_range_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=0
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${port_range_bad_dir}" >"${tmp_dir}/port-range-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on out-of-range metadata port" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt port must be between 1 and 65535' "${tmp_dir}/port-range-bad.log"; then
  echo "expected metadata-range diagnostic for metadata port" >&2
  exit 1
fi

timeout_range_bad_dir="${tmp_dir}/bad-timeout-range"
cp -R "${ok_dir}" "${timeout_range_bad_dir}"
cat > "${timeout_range_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=0
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${timeout_range_bad_dir}" >"${tmp_dir}/timeout-range-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on non-positive metadata timeout" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt serveTimeoutMs must be greater than 0' "${tmp_dir}/timeout-range-bad.log"; then
  echo "expected metadata-range diagnostic for metadata timeout" >&2
  exit 1
fi

max_body_missing_dir="${tmp_dir}/bad-max-body-missing"
cp -R "${ok_dir}" "${max_body_missing_dir}"
cat > "${max_body_missing_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${max_body_missing_dir}" >"${tmp_dir}/max-body-missing.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing maxBodyBytes metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing or invalid maxBodyBytes field' "${tmp_dir}/max-body-missing.log"; then
  echo "expected missing-or-invalid diagnostic for maxBodyBytes metadata" >&2
  exit 1
fi

max_body_range_bad_dir="${tmp_dir}/bad-max-body-range"
cp -R "${ok_dir}" "${max_body_range_bad_dir}"
cat > "${max_body_range_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=0
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${max_body_range_bad_dir}" >"${tmp_dir}/max-body-range-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on non-positive maxBodyBytes metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt maxBodyBytes must be unset or positive integer' "${tmp_dir}/max-body-range-bad.log"; then
  echo "expected range diagnostic for maxBodyBytes metadata" >&2
  exit 1
fi

max_body_log_mismatch_dir="${tmp_dir}/bad-max-body-log-mismatch"
cp -R "${ok_dir}" "${max_body_log_mismatch_dir}"
cat > "${max_body_log_mismatch_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=2048
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${max_body_log_mismatch_dir}" >"${tmp_dir}/max-body-log-mismatch.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail when max-body metadata and run logs diverge" >&2
  exit 1
fi

if ! rg -Fq 'health.run.log missing --max-body-bytes 2048 invocation token' "${tmp_dir}/max-body-log-mismatch.log"; then
  echo "expected max-body correlation diagnostic for health run log" >&2
  exit 1
fi

meta_bad_dir="${tmp_dir}/bad-metadata"
cp -R "${ok_dir}" "${meta_bad_dir}"
cat > "${meta_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${meta_bad_dir}" >"${tmp_dir}/meta-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing oneshot metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing oneshot=true field' "${tmp_dir}/meta-bad.log"; then
  echo "expected metadata diagnostic for missing oneshot field" >&2
  exit 1
fi

source_bad_dir="${tmp_dir}/bad-source"
cp -R "${ok_dir}" "${source_bad_dir}"
cat > "${source_bad_dir}/run-metadata.txt" <<'TXT'
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${source_bad_dir}" >"${tmp_dir}/source-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing sourceProject metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing sourceProject field' "${tmp_dir}/source-bad.log"; then
  echo "expected metadata diagnostic for missing sourceProject field" >&2
  exit 1
fi

source_empty_dir="${tmp_dir}/bad-source-empty"
cp -R "${ok_dir}" "${source_empty_dir}"
cat > "${source_empty_dir}/run-metadata.txt" <<'TXT'
sourceProject=
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${source_empty_dir}" >"${tmp_dir}/source-empty.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on empty sourceProject metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing sourceProject field' "${tmp_dir}/source-empty.log"; then
  echo "expected metadata diagnostic for empty sourceProject field" >&2
  exit 1
fi

work_bad_dir="${tmp_dir}/bad-work"
cp -R "${ok_dir}" "${work_bad_dir}"
cat > "${work_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${work_bad_dir}" >"${tmp_dir}/work-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing workProject metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing workProject field' "${tmp_dir}/work-bad.log"; then
  echo "expected metadata diagnostic for missing workProject field" >&2
  exit 1
fi

work_empty_dir="${tmp_dir}/bad-work-empty"
cp -R "${ok_dir}" "${work_empty_dir}"
cat > "${work_empty_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${work_empty_dir}" >"${tmp_dir}/work-empty.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on empty workProject metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing workProject field' "${tmp_dir}/work-empty.log"; then
  echo "expected metadata diagnostic for empty workProject field" >&2
  exit 1
fi

same_project_dir="${tmp_dir}/bad-same-project"
cp -R "${ok_dir}" "${same_project_dir}"
cat > "${same_project_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=examples/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${checker}" --artifacts-dir "${same_project_dir}" >"${tmp_dir}/same-project.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail when source/work metadata are identical" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt sourceProject and workProject must differ' "${tmp_dir}/same-project.log"; then
  echo "expected metadata diagnostic when source/work metadata values are identical" >&2
  exit 1
fi

runflags_bad_dir="${tmp_dir}/bad-runflags"
cp -R "${ok_dir}" "${runflags_bad_dir}"
cat > "${runflags_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
TXT

if "${checker}" --artifacts-dir "${runflags_bad_dir}" >"${tmp_dir}/runflags-bad.log" 2>&1; then
  echo "expected runtime-smoke artifacts checker to fail on missing runFlags metadata" >&2
  exit 1
fi

if ! rg -Fq 'run-metadata.txt missing deterministic runFlags field' "${tmp_dir}/runflags-bad.log"; then
  echo "expected metadata diagnostic for missing runFlags field" >&2
  exit 1
fi

echo "runtime-smoke artifacts checker test passed"
