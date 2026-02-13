#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

matrix_ok="${root_dir}/testdata/sample-cross-impl-compare-matrix.json"
matrix_bad="${root_dir}/testdata/sample-compare-matrix.json"
target="${tmp}/compare-matrix.json"

dry_out="$("$root_dir/update_cross_impl_matrix_from_ci.sh" --dry-run --repo HubDev-AI/untrusted-compiler --out-dir "$tmp")"
if ! grep -q 'gh run list --repo HubDev-AI/untrusted-compiler --workflow benchmark-cross-impl-evidence.yml' <<<"$dry_out"; then
  echo "dry-run missing gh run list command" >&2
  exit 1
fi
if ! grep -q 'validate matrix includes impls sec4,go,node,rust' <<<"$dry_out"; then
  echo "dry-run missing validation step" >&2
  exit 1
fi

"$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$matrix_ok" --target "$target" >/dev/null
if [ ! -f "$target" ]; then
  echo "target matrix not written" >&2
  exit 1
fi
if ! jq -e '.endpoints[0].compared[] | select(.impl == "node")' "$target" >/dev/null; then
  echo "expected node impl in imported matrix" >&2
  exit 1
fi

if "$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$matrix_bad" --target "$target" >/dev/null 2>&1; then
  echo "expected failure for matrix missing required impls" >&2
  exit 1
fi

echo "update_cross_impl_matrix_from_ci test passed"
