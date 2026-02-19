#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/check-zed-extension-release.sh [--skip-smoke]

Validates Zed extension release readiness:
- required extension files exist
- extension manifest wiring is present
- grammar pin is valid
- plugin smoke runner passes (fast mode) unless skipped
USAGE
}

skip_smoke="0"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-smoke)
      skip_smoke="1"
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

required_files=(
  "zed-extension/extension.toml"
  "zed-extension/languages/untrusted/config.toml"
  "zed-extension/src/lib.rs"
  "zed-extension/README.md"
  "examples/zed-plugin-smoke/README.md"
)

for required in "${required_files[@]}"; do
  if [[ ! -f "${repo_root}/${required}" ]]; then
    echo "zed extension release check failed: missing ${required}" >&2
    exit 1
  fi
done

manifest="${repo_root}/zed-extension/extension.toml"
required_tokens=(
  'id = "untrusted"'
  'name = "Untrusted<T>"'
  'schema_version = 1'
  '[grammars.untrusted]'
  '[language_servers.sec4audit-lsp]'
  'languages = ["Untrusted<T>"]'
)
for token in "${required_tokens[@]}"; do
  if ! rg -Fq -- "${token}" "${manifest}"; then
    echo "zed extension release check failed: missing token ${token} in zed-extension/extension.toml" >&2
    exit 1
  fi
done

"${repo_root}/scripts/check-zed-grammar-pin.sh"

if [[ "${skip_smoke}" != "1" ]]; then
  "${repo_root}/scripts/run-zed-plugin-smoke.sh" --fast
fi

echo "zed extension release check passed"
