#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$($root_dir/scripts/preflight.sh --impls ailang,node --dry-run-only)"
if ! grep -q '^OK       HTTP probing (curl)$' <<<"$out"; then
  echo "preflight missing curl check" >&2
  exit 1
fi
if ! grep -q '^OK       JSON processing (jq)$' <<<"$out"; then
  echo "preflight missing jq check" >&2
  exit 1
fi
if ! grep -q 'AILang compiler runner' <<<"$out"; then
  echo "preflight missing ailang tool check" >&2
  exit 1
fi

out_equals="$($root_dir/scripts/preflight.sh --impls=ailang,node --dry-run-only)"
if ! grep -q '^preflight passed$' <<<"$out_equals"; then
  echo "preflight did not pass with --impls= syntax" >&2
  exit 1
fi

if "$root_dir/scripts/preflight.sh" --impls unknown --dry-run-only >/dev/null 2>&1; then
  echo "expected unknown implementation to fail" >&2
  exit 1
fi

echo "preflight test passed"
