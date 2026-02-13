#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check_script="${root_dir}/scripts/check-zed-grammar-pin.sh"
extension_toml="${root_dir}/zed-extension/extension.toml"
expected_repo="https://github.com/HubDev-AI/untrusted-compiler"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --extension)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --extension" >&2
        exit 2
      fi
      extension_toml="$2"
      shift 2
      ;;
    --extension=*)
      extension_toml="${1#--extension=}"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "${check_script}" ]]; then
  echo "missing grammar pin checker: ${check_script}" >&2
  exit 1
fi
if [[ ! -f "${extension_toml}" ]]; then
  echo "missing extension.toml: ${extension_toml}" >&2
  exit 1
fi

"${check_script}" --extension "${extension_toml}" >/dev/null

if ! rg -q "^[[:space:]]*repository[[:space:]]*=[[:space:]]*\"${expected_repo}\"[[:space:]]*$" "${extension_toml}"; then
  echo "missing pinned grammar repository (${expected_repo}) in ${extension_toml}" >&2
  exit 1
fi

echo "zed grammar pin contract test passed"
