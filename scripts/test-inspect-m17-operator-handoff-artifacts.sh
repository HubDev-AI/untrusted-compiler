#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
inspect_script="${root_dir}/scripts/inspect-m17-operator-handoff-artifacts.sh"

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

text_output="$("${inspect_script}" --repo-root "${root_dir}" --artifacts-root "${ok_root}")"
if ! printf '%s\n' "${text_output}" | rg -Fq 'M17 Operator Handoff Artifact Summary'; then
  echo "expected artifact inspector text heading" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'branch=default'; then
  echo "expected default branch row in artifact inspector text output" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'branch=max-body'; then
  echo "expected max-body branch row in artifact inspector text output" >&2
  exit 1
fi

json_output="$("${inspect_script}" --repo-root "${root_dir}" --artifacts-root "${ok_root}" --format json)"
if ! printf '%s\n' "${json_output}" | jq -e '
  .version == "0.1"
  and .branchCount == 2
  and [.branches[].name] == ["default","max-body"]
  and (.branches[] | select(.name == "max-body")).maxBodyBytes == "2048"
' >/dev/null; then
  echo "expected deterministic artifact inspector json output contract" >&2
  exit 1
fi

missing_branch_root="${tmp_dir}/bad-missing-branch"
cp -R "${ok_root}" "${missing_branch_root}"
rm -rf "${missing_branch_root}/max-body"

if "${inspect_script}" --repo-root "${root_dir}" --artifacts-root "${missing_branch_root}" >"${tmp_dir}/missing-branch.log" 2>&1; then
  echo "expected artifact inspector to fail when max-body branch is missing" >&2
  exit 1
fi
if ! rg -Fq 'runtime-smoke bundle missing branch directory: max-body' "${tmp_dir}/missing-branch.log"; then
  echo "expected missing-branch diagnostic from artifact inspector" >&2
  exit 1
fi

echo "m17 operator handoff artifact inspector test passed"
