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

ok_root="${tmp_dir}/ok"
write_branch_fixture "${ok_root}" "default" "unset" "--port,--oneshot,--serve-timeout-ms" "rt-1" "8080" "false"
write_branch_fixture "${ok_root}" "max-body" "2048" "--port,--oneshot,--serve-timeout-ms,--max-body-bytes" "rt-2" "8081" "true"

ok_index="${ok_root}/runtime-smoke-branch-index.json"
"${bundle_checker}" --artifacts-root "${ok_root}" --index-path "${ok_index}" >/dev/null

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

echo "runtime-smoke bundle checker test passed"
