#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
index_builder="${root_dir}/scripts/build-runtime-smoke-branch-index.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

write_branch_fixture() {
  local root="$1"
  local branch="$2"
  local max_body="$3"
  local run_flags="$4"
  local trace_id="$5"
  local port="$6"

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

  cat > "${branch_dir}/users.headers" <<TXT
HTTP/1.1 201 Created
Content-Type: application/json; charset=utf-8
X-Trace-Id: ${trace_id}
TXT

  cat > "${branch_dir}/users.body" <<TXT
{"ok":true,"userId":"123e4567-e89b-42d3-a456-426614174000"}
TXT
}

ok_root="${tmp_dir}/ok"
write_branch_fixture "${ok_root}" "default" "unset" "--port,--oneshot,--serve-timeout-ms" "rt-1" "8080"
write_branch_fixture "${ok_root}" "max-body" "2048" "--port,--oneshot,--serve-timeout-ms,--max-body-bytes" "rt-2" "8081"

ok_index="${tmp_dir}/runtime-smoke-branch-index.json"
"${index_builder}" --artifacts-root "${ok_root}" --out "${ok_index}" >/dev/null

if ! jq -e '.version == "0.1" and .branchOrder == ["default","max-body"] and (.branches | length == 2)' "${ok_index}" >/dev/null; then
  echo "expected runtime-smoke branch index to include version, branch order, and two branches" >&2
  exit 1
fi

if ! jq -e '.branches[] | select(.branch == "default") | .maxBodyBytes == "unset"' "${ok_index}" >/dev/null; then
  echo "expected default branch maxBodyBytes=unset in branch index" >&2
  exit 1
fi

if ! jq -e '.branches[] | select(.branch == "max-body") | .maxBodyBytes == "2048"' "${ok_index}" >/dev/null; then
  echo "expected max-body branch maxBodyBytes=2048 in branch index" >&2
  exit 1
fi

missing_branch_root="${tmp_dir}/bad-missing-branch"
cp -R "${ok_root}" "${missing_branch_root}"
rm -rf "${missing_branch_root}/max-body"

if "${index_builder}" --artifacts-root "${missing_branch_root}" --out "${tmp_dir}/missing-branch-index.json" >"${tmp_dir}/missing-branch.log" 2>&1; then
  echo "expected runtime-smoke branch index builder to fail on missing max-body branch" >&2
  exit 1
fi

if ! rg -Fq 'missing runtime-smoke branch artifacts directory: max-body' "${tmp_dir}/missing-branch.log"; then
  echo "expected missing-branch diagnostic for max-body artifacts directory" >&2
  exit 1
fi

default_shape_root="${tmp_dir}/bad-default-shape"
cp -R "${ok_root}" "${default_shape_root}"
cat > "${default_shape_root}/default/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api-default
port=8080
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=2048
runFlags=--port,--oneshot,--serve-timeout-ms,--max-body-bytes
TXT

if "${index_builder}" --artifacts-root "${default_shape_root}" --out "${tmp_dir}/default-shape-index.json" >"${tmp_dir}/default-shape.log" 2>&1; then
  echo "expected runtime-smoke branch index builder to fail when default branch has numeric maxBodyBytes" >&2
  exit 1
fi

if ! rg -Fq 'runtime-smoke index expects default branch maxBodyBytes=unset' "${tmp_dir}/default-shape.log"; then
  echo "expected default-branch maxBody diagnostic" >&2
  exit 1
fi

max_body_shape_root="${tmp_dir}/bad-max-body-shape"
cp -R "${ok_root}" "${max_body_shape_root}"
cat > "${max_body_shape_root}/max-body/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
workProject=/tmp/hello-api-max-body
port=8081
oneshot=true
serveTimeoutMs=12000
maxBodyBytes=unset
runFlags=--port,--oneshot,--serve-timeout-ms
TXT

if "${index_builder}" --artifacts-root "${max_body_shape_root}" --out "${tmp_dir}/max-body-shape-index.json" >"${tmp_dir}/max-body-shape.log" 2>&1; then
  echo "expected runtime-smoke branch index builder to fail when max-body branch has unset maxBodyBytes" >&2
  exit 1
fi

if ! rg -Fq 'runtime-smoke index expects max-body branch maxBodyBytes to be numeric' "${tmp_dir}/max-body-shape.log"; then
  echo "expected max-body branch maxBody diagnostic" >&2
  exit 1
fi

echo "runtime-smoke branch index builder test passed"
