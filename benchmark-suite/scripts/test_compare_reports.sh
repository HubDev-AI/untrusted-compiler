#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-ailang-report.json" "$tmp/ailang-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"

out="$tmp/compare.json"
"$root_dir/scripts/compare_reports.sh" "$tmp" ping "$out" >/dev/null

if ! grep -q '"endpoint": "ping"' "$out"; then
  echo "missing endpoint in compare output" >&2
  exit 1
fi
if ! grep -q '"impl": "go"' "$out"; then
  echo "missing go row in compare output" >&2
  exit 1
fi
if ! grep -q '"impl": "ailang"' "$out"; then
  echo "missing ailang row in compare output" >&2
  exit 1
fi
if ! grep -q '"leader": {' "$out"; then
  echo "missing leader object" >&2
  exit 1
fi
if ! grep -q '"impl": "go"' <(jq '.leader' "$out"); then
  echo "expected go leader by requests/sec" >&2
  exit 1
fi

echo "compare_reports test passed"
