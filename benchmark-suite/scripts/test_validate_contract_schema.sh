#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$root_dir/validate_contract_schema.sh" >/dev/null

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

mkdir -p "$tmp/samples" "$tmp/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp/schemas/"

"$root_dir/validate_contract_schema.sh" --samples-dir "$tmp/samples" --schema-dir "$tmp/schemas" >/dev/null

jq 'del(.latency)' "$tmp/samples/sample-summary-ping.json" > "$tmp/samples/sample-summary-ping.json.tmp"
mv "$tmp/samples/sample-summary-ping.json.tmp" "$tmp/samples/sample-summary-ping.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp/samples" --schema-dir "$tmp/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail after removing required key" >&2
  exit 1
fi

tmp2="$(mktemp -d)"
cleanup2() {
  rm -rf "$tmp2"
}
trap 'cleanup; cleanup2' EXIT

mkdir -p "$tmp2/samples" "$tmp2/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp2/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp2/schemas/"

jq 'del(.endpoints[0].leader.constantRate)' "$tmp2/samples/sample-compare-matrix.json" > "$tmp2/samples/sample-compare-matrix.json.tmp"
mv "$tmp2/samples/sample-compare-matrix.json.tmp" "$tmp2/samples/sample-compare-matrix.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp2/samples" --schema-dir "$tmp2/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail after removing compare-matrix row quality key" >&2
  exit 1
fi

echo "validate_contract_schema test passed"
