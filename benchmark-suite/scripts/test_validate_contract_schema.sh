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

tmp3="$(mktemp -d)"
cleanup3() {
  rm -rf "$tmp3"
}
trap 'cleanup; cleanup2; cleanup3' EXIT

mkdir -p "$tmp3/samples" "$tmp3/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp3/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp3/schemas/"

jq 'del(.leader.loadGenerator)' "$tmp3/samples/sample-compare-report-ping.json" > "$tmp3/samples/sample-compare-report-ping.json.tmp"
mv "$tmp3/samples/sample-compare-report-ping.json.tmp" "$tmp3/samples/sample-compare-report-ping.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp3/samples" --schema-dir "$tmp3/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail after removing compare-report row quality key" >&2
  exit 1
fi

tmp4="$(mktemp -d)"
cleanup4() {
  rm -rf "$tmp4"
}
trap 'cleanup; cleanup2; cleanup3; cleanup4' EXIT

mkdir -p "$tmp4/samples" "$tmp4/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp4/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp4/schemas/"

jq '.leader.impl = "rust"' "$tmp4/samples/sample-compare-report-ping.json" > "$tmp4/samples/sample-compare-report-ping.json.tmp"
mv "$tmp4/samples/sample-compare-report-ping.json.tmp" "$tmp4/samples/sample-compare-report-ping.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp4/samples" --schema-dir "$tmp4/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail when compare-report leader is not present in compared rows" >&2
  exit 1
fi

tmp5="$(mktemp -d)"
cleanup5() {
  rm -rf "$tmp5"
}
trap 'cleanup; cleanup2; cleanup3; cleanup4; cleanup5' EXIT

mkdir -p "$tmp5/samples" "$tmp5/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp5/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp5/schemas/"

jq '.endpoints[0].compared[0].endpoint = "decode"' "$tmp5/samples/sample-compare-matrix.json" > "$tmp5/samples/sample-compare-matrix.json.tmp"
mv "$tmp5/samples/sample-compare-matrix.json.tmp" "$tmp5/samples/sample-compare-matrix.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp5/samples" --schema-dir "$tmp5/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail when compare-matrix row endpoint mismatches endpoint group" >&2
  exit 1
fi

tmp6="$(mktemp -d)"
cleanup6() {
  rm -rf "$tmp6"
}
trap 'cleanup; cleanup2; cleanup3; cleanup4; cleanup5; cleanup6' EXIT

mkdir -p "$tmp6/samples" "$tmp6/schemas"
cp "$root_dir"/testdata/sample-*.json "$tmp6/samples/"
cp "$root_dir"/../spec/schemas/*.schema.json "$tmp6/schemas/"

jq 'del(.memory.rssKb)' "$tmp6/samples/sample-summary-ping.json" > "$tmp6/samples/sample-summary-ping.json.tmp"
mv "$tmp6/samples/sample-summary-ping.json.tmp" "$tmp6/samples/sample-summary-ping.json"

if "$root_dir/validate_contract_schema.sh" --samples-dir "$tmp6/samples" --schema-dir "$tmp6/schemas" >/dev/null 2>&1; then
  echo "expected schema validator to fail when summary memory rssKb is missing" >&2
  exit 1
fi

echo "validate_contract_schema test passed"
