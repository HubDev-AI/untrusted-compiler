#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="${root_dir}/scripts/check-no-local-path-leaks.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

mkdir -p "${tmp}/docs"
cat > "${tmp}/docs/ok.md" <<'TXT'
# Safe doc
No local absolute paths here.
TXT

"${checker}" --repo-root "${tmp}" >/dev/null

forbidden_prefix="/Users/""vladimirtrifonov""/src/ai/"
printf 'Path leak: %sproject\n' "${forbidden_prefix}" > "${tmp}/docs/leak.md"

if "${checker}" --repo-root "${tmp}" >/dev/null 2>&1; then
  echo "expected local path leak checker to fail on forbidden token" >&2
  exit 1
fi

echo "local path leak checker test passed"
