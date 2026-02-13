#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

out="$("$root_dir/fetch_trend_artifact.sh" \
  --dry-run \
  --repo HubDev-AI/untrusted-compiler \
  --workflow benchmark-trend.yml \
  --artifact-name benchmark-trend-node-ping-decode \
  --out-dir "$tmp")"

if ! grep -q 'gh run list --repo HubDev-AI/untrusted-compiler --workflow benchmark-trend.yml' <<<"$out"; then
  echo "missing gh run list command in dry-run output" >&2
  exit 1
fi

if ! grep -q "gh run download <run_id> --repo HubDev-AI/untrusted-compiler --name benchmark-trend-node-ping-decode --dir $tmp" <<<"$out"; then
  echo "missing gh run download command in dry-run output" >&2
  exit 1
fi

echo "fetch_trend_artifact test passed"
