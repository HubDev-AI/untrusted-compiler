#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/summaries"
cp "$root_dir/scripts/testdata/sample-summary-ping.json" "$tmp/summaries/sec4-ping.json"
cp "$root_dir/scripts/testdata/sample-summary-decode.json" "$tmp/summaries/sec4-decode.json"
cp "$root_dir/scripts/testdata/sample-env.json" "$tmp/env.json"
cp "$root_dir/scripts/testdata/sample-summary-decode.json" "$tmp/summaries/sec4-users-post.json"

out="$tmp/report.json"
"$root_dir/scripts/build_report.sh" sec4 "$tmp" "$out" >/dev/null

if ! grep -q '"impl": "sec4"' "$out"; then
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

filtered_out="$tmp/report-filtered.json"
"$root_dir/scripts/build_report.sh" sec4 "$tmp" "$filtered_out" "" "ping" >/dev/null
if ! grep -q '"endpoint": "ping"' "$filtered_out"; then
  echo "missing ping summary in filtered report" >&2
  exit 1
fi
if grep -q '"endpoint": "decode"' "$filtered_out"; then
  echo "filtered report should not include decode summary" >&2
  exit 1
fi
if [ "$(jq -r '.selectedEndpoints[0] // empty' "$filtered_out")" != "ping" ]; then
  echo "filtered report selectedEndpoints missing ping" >&2
  exit 1
fi

echo "build_report test passed"
