#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/summaries"
cp "$root_dir/scripts/testdata/sample-summary-ping.json" "$tmp/summaries/ailang-ping.json"
cp "$root_dir/scripts/testdata/sample-summary-decode.json" "$tmp/summaries/ailang-decode.json"
cp "$root_dir/scripts/testdata/sample-env.json" "$tmp/env.json"

out="$tmp/report.json"
"$root_dir/scripts/build_report.sh" ailang "$tmp" "$out" >/dev/null

if ! grep -q '"impl": "ailang"' "$out"; then
  echo "missing impl in report" >&2
  exit 1
fi
if ! grep -q '"endpoint": "decode"' "$out"; then
  echo "missing decode summary in report" >&2
  exit 1
fi
if ! grep -q '"cpu": "test-cpu"' "$out"; then
  echo "missing env payload in report" >&2
  exit 1
fi

echo "build_report test passed"
