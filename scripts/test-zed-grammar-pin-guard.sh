#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-zed-grammar-pin.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

extension_toml="${tmp}/extension.toml"

cat > "${extension_toml}" <<'TOML'
id = "untrusted"
name = "Untrusted<T>"
version = "0.1.0"
schema_version = 1

[grammars.untrusted]
repository = "https://github.com/HubDev-AI/untrusted-compiler"
rev = "e5ab04b3d433f87d57d090127d056fb78df5d38b"
TOML

"${contract_script}" --extension "${extension_toml}" >/dev/null

cat > "${extension_toml}" <<'TOML'
id = "untrusted"
name = "Untrusted<T>"
version = "0.1.0"
schema_version = 1

[grammars.untrusted]
repository = "https://github.com/HubDev-AI/untrusted-compiler"
rev = "TODO_SET_COMMIT_SHA"
TOML

if "${contract_script}" --extension "${extension_toml}" >/dev/null 2>&1; then
  echo "expected grammar pin contract failure for placeholder revision" >&2
  exit 1
fi

cat > "${extension_toml}" <<'TOML'
id = "untrusted"
name = "Untrusted<T>"
version = "0.1.0"
schema_version = 1

[grammars.untrusted]
repository = "https://github.com/HubDev-AI/untrusted-compiler"
rev = "e5ab04b"
TOML

if "${contract_script}" --extension "${extension_toml}" >/dev/null 2>&1; then
  echo "expected grammar pin contract failure for short SHA revision" >&2
  exit 1
fi

cat > "${extension_toml}" <<'TOML'
id = "untrusted"
name = "Untrusted<T>"
version = "0.1.0"
schema_version = 1

[grammars.untrusted]
repository = "https://github.com/HubDev-AI/untrusted-compiler"
rev = "not-a-sha"
TOML

if "${contract_script}" --extension "${extension_toml}" >/dev/null 2>&1; then
  echo "expected grammar pin contract failure for non-SHA revision" >&2
  exit 1
fi

cat > "${extension_toml}" <<'TOML'
id = "untrusted"
name = "Untrusted<T>"
version = "0.1.0"
schema_version = 1

[grammars.untrusted]
repository = "https://github.com/example/wrong-repo"
rev = "e5ab04b3d433f87d57d090127d056fb78df5d38b"
TOML

if "${contract_script}" --extension "${extension_toml}" >/dev/null 2>&1; then
  echo "expected grammar pin contract failure for repository drift" >&2
  exit 1
fi

echo "zed grammar pin guard test passed"
