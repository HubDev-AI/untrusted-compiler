#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract.sh"
source_script="${root_dir}/scripts/smoke-sec4-run-lasm-db-adapter.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

fixture="${tmp_dir}/smoke.sh"
cp "${source_script}" "${fixture}"
chmod +x "${fixture}"

"${contract_script}" --script "${fixture}" >/dev/null

missing_auth_token='default_smoke_auth_header="Authorization: Bearer smoke-token"'
perl -0pi -e 's/default_smoke_auth_header="[^"]*"/default_smoke_auth_header="Authorization: removed-token"/' "${fixture}"

if "${contract_script}" --script "${fixture}" >"${tmp_dir}/auth-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when authorization token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_auth_token}" "${tmp_dir}/auth-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_auth_token}" >&2
  exit 1
fi

meta_fixture="${tmp_dir}/smoke-meta.sh"
cp "${source_script}" "${meta_fixture}"
chmod +x "${meta_fixture}"

missing_meta_token='dbAdapterLabel=${adapter_label}'
perl -0pi -e 's/dbAdapterLabel=\$\{adapter_label\}/dbAdapterLabel=removed-label/' "${meta_fixture}"

if "${contract_script}" --script "${meta_fixture}" >"${tmp_dir}/meta-guard.log" 2>&1; then
  echo "expected smoke script contract to fail when dbAdapterLabel metadata token is removed" >&2
  exit 1
fi

if ! rg -Fq "missing required smoke-script token: ${missing_meta_token}" "${tmp_dir}/meta-guard.log"; then
  echo "expected missing-token diagnostic for ${missing_meta_token}" >&2
  exit 1
fi

echo "sec4 run lasm db-adapter smoke script contract guard test passed"
