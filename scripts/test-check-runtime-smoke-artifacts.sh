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
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

cat > "${ok_dir}/health.headers" <<'TXT'
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
TXT

cat > "${ok_dir}/health.body" <<'TXT'
ok
TXT

cat > "${ok_dir}/health.run.log" <<'TXT'
smoke log health
TXT

cat > "${ok_dir}/users.headers" <<'TXT'
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
TXT

cat > "${ok_dir}/users.body" <<'TXT'
{"ok":true,"status":201,"traceId":"rt-1","timeMs":1,"data":1}
TXT

cat > "${ok_dir}/users.run.log" <<'TXT'
smoke log users
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

meta_bad_dir="${tmp_dir}/bad-metadata"
cp -R "${ok_dir}" "${meta_bad_dir}"
cat > "${meta_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
serveTimeoutMs=12000
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

work_bad_dir="${tmp_dir}/bad-work"
cp -R "${ok_dir}" "${work_bad_dir}"
cat > "${work_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
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

runflags_bad_dir="${tmp_dir}/bad-runflags"
cp -R "${ok_dir}" "${runflags_bad_dir}"
cat > "${runflags_bad_dir}/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api
port=8080
oneshot=true
serveTimeoutMs=12000
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
