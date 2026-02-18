#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$($root_dir/scripts/test_service_contracts.sh --dry-run --impls node,c)"

if ! grep -q '/services/node/smoke.sh' <<<"$out"; then
  echo "missing node smoke command" >&2
  exit 1
fi
if ! grep -q '/services/c/smoke.sh' <<<"$out"; then
  echo "missing c smoke command" >&2
  exit 1
fi

lasm_out="$($root_dir/scripts/test_service_contracts.sh --dry-run --impls sec4-lasm)"
if ! grep -q '/services/sec4-lasm/smoke.sh' <<<"$lasm_out"; then
  echo "missing sec4-lasm smoke command" >&2
  exit 1
fi

equals_out="$($root_dir/scripts/test_service_contracts.sh --dry-run --impls=node,c)"
if ! grep -q '/services/node/smoke.sh' <<<"$equals_out"; then
  echo "missing node smoke command for --impls= syntax" >&2
  exit 1
fi

if "$root_dir/scripts/test_service_contracts.sh" --dry-run --impls unknown >/dev/null 2>&1; then
  echo "expected unknown impl validation failure" >&2
  exit 1
fi

echo "test_service_contracts test passed"
