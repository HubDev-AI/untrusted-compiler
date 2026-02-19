#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/run-zed-plugin-smoke.sh [--fast]

Runs deterministic local checks for the Zed plugin workflow using examples/zed-plugin-smoke.

--fast  Skip targeted sec4audit-language-server formatting tests.
USAGE
}

fast="${FAST:-0}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fast)
      fast="1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
smoke_src="${repo_root}/examples/zed-plugin-smoke"

if [[ ! -d "${smoke_src}" ]]; then
  echo "zed smoke failed: missing ${smoke_src}" >&2
  exit 1
fi

cd "${repo_root}"

# 1) Keep grammar pin guard green.
"${repo_root}/scripts/check-zed-grammar-pin.sh"

# 2) Validate sample project compiles/analyzes.
cargo run -p sec4 -- check --path "${smoke_src}"

# 3) Verify formatter behavior in a temp copy (do not mutate tracked fixtures).
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/sec4-zed-smoke.XXXXXX")"
trap 'rm -rf "${tmp_root}"' EXIT

cp -R "${smoke_src}" "${tmp_root}/project"
cargo run -p sec4 -- fmt --path "${tmp_root}/project"

expected_file="${tmp_root}/expected-format.ut"
cat > "${expected_file}" <<'EXPECTED'
fn sample() -> Int {
  0
}
EXPECTED

if ! diff -u "${expected_file}" "${tmp_root}/project/playground/format-me.ut" >/dev/null; then
  echo "zed smoke failed: formatter output mismatch for playground/format-me.ut" >&2
  diff -u "${expected_file}" "${tmp_root}/project/playground/format-me.ut" >&2 || true
  exit 1
fi

# 4) Optionally verify LSP formatting tests directly.
if [[ "${fast}" != "1" ]]; then
  cargo test -p sec4audit-language-server formatting_request_returns_full_document_edit_when_source_needs_changes -- --exact
  cargo test -p sec4audit-language-server formatting_request_returns_no_edits_when_source_is_already_formatted -- --exact
fi

echo "zed plugin smoke passed"
echo "manual zed checks: ${smoke_src}/README.md"
