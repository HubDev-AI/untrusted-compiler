#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-smoke-sec4-run-hello-api-script-contract.sh"
source_script="${root_dir}/scripts/smoke-sec4-run-hello-api.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

fixture="${tmp_dir}/smoke.sh"
cp "${source_script}" "${fixture}"
chmod +x "${fixture}"

"${contract_script}" --script "${fixture}" >/dev/null

missing_token="X-CSRF-Token: token123"
perl -0pi -e 's/X-CSRF-Token: token123/X-CSRF-Token: removed-token/' "${fixture}"

if "${contract_script}" --script "${fixture}" >"${tmp_dir}/guard.log" 2>&1; then
  echo "expected smoke script contract to fail when a required token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_token}" "${tmp_dir}/guard.log"; then
  echo "expected missing-token diagnostic for ${missing_token}" >&2
  exit 1
fi

meta_fixture="${tmp_dir}/smoke-meta.sh"
cp "${source_script}" "${meta_fixture}"
chmod +x "${meta_fixture}"

missing_source_meta_token='sourceProject=${source_project}'
perl -0pi -e 's/sourceProject=\$\{source_project\}/sourceProject=removed-source/' "${meta_fixture}"

if "${contract_script}" --script "${meta_fixture}" >"${tmp_dir}/source-meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when sourceProject metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_source_meta_token}" "${tmp_dir}/source-meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_source_meta_token}" >&2
  exit 1
fi

meta_work_fixture="${tmp_dir}/smoke-meta-work.sh"
cp "${source_script}" "${meta_work_fixture}"
chmod +x "${meta_work_fixture}"

missing_work_meta_token='workProject=${work_project}'
perl -0pi -e 's/workProject=\$\{work_project\}/workProject=removed-work/' "${meta_work_fixture}"

if "${contract_script}" --script "${meta_work_fixture}" >"${tmp_dir}/work-meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when workProject metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_work_meta_token}" "${tmp_dir}/work-meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_work_meta_token}" >&2
  exit 1
fi

timeout_flag_fixture="${tmp_dir}/smoke-timeout-flag.sh"
cp "${source_script}" "${timeout_flag_fixture}"
chmod +x "${timeout_flag_fixture}"

missing_timeout_flag_token='--serve-timeout-ms "${serve_timeout_ms}"'
perl -0pi -e 's/--serve-timeout-ms "\$\{serve_timeout_ms\}"/--serve-timeout-ms removed-timeout/' "${timeout_flag_fixture}"

if "${contract_script}" --script "${timeout_flag_fixture}" >"${tmp_dir}/timeout-flag-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when serve-timeout flag token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_timeout_flag_token}" "${tmp_dir}/timeout-flag-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_timeout_flag_token}" >&2
  exit 1
fi

timeout_meta_fixture="${tmp_dir}/smoke-timeout-meta.sh"
cp "${source_script}" "${timeout_meta_fixture}"
chmod +x "${timeout_meta_fixture}"

missing_timeout_meta_token='serveTimeoutMs=${serve_timeout_ms}'
perl -0pi -e 's/serveTimeoutMs=\$\{serve_timeout_ms\}/serveTimeoutMs=removed-timeout/' "${timeout_meta_fixture}"

if "${contract_script}" --script "${timeout_meta_fixture}" >"${tmp_dir}/timeout-meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when serveTimeoutMs metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_timeout_meta_token}" "${tmp_dir}/timeout-meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_timeout_meta_token}" >&2
  exit 1
fi

max_body_flag_fixture="${tmp_dir}/smoke-max-body-flag.sh"
cp "${source_script}" "${max_body_flag_fixture}"
chmod +x "${max_body_flag_fixture}"

missing_max_body_flag_token='--max-body-bytes "${max_body_bytes}"'
perl -0pi -e 's/--max-body-bytes "\$\{max_body_bytes\}"/--max-body-bytes removed-max-body/' "${max_body_flag_fixture}"

if "${contract_script}" --script "${max_body_flag_fixture}" >"${tmp_dir}/max-body-flag-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when max-body flag token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_max_body_flag_token}" "${tmp_dir}/max-body-flag-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_max_body_flag_token}" >&2
  exit 1
fi

max_body_meta_fixture="${tmp_dir}/smoke-max-body-meta.sh"
cp "${source_script}" "${max_body_meta_fixture}"
chmod +x "${max_body_meta_fixture}"

missing_max_body_meta_token='maxBodyBytes=${max_body_bytes:-unset}'
perl -0pi -e 's/maxBodyBytes=\$\{max_body_bytes:-unset\}/maxBodyBytes=removed-max-body/' "${max_body_meta_fixture}"

if "${contract_script}" --script "${max_body_meta_fixture}" >"${tmp_dir}/max-body-meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when maxBodyBytes metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_max_body_meta_token}" "${tmp_dir}/max-body-meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_max_body_meta_token}" >&2
  exit 1
fi

runflags_fixture="${tmp_dir}/smoke-meta-runflags.sh"
cp "${source_script}" "${runflags_fixture}"
chmod +x "${runflags_fixture}"

missing_meta_token='runFlags=${run_flags}'
perl -0pi -e 's/runFlags=\$\{run_flags\}/runFlags=removed-flags/' "${runflags_fixture}"

if "${contract_script}" --script "${runflags_fixture}" >"${tmp_dir}/meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_meta_token}" "${tmp_dir}/meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_meta_token}" >&2
  exit 1
fi

echo "sec4 run hello-api smoke script contract guard test passed"
