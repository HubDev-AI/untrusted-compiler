#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/results/summaries" "$tmp/results/raw"
printf 'alpha\n' > "$tmp/results/summaries/a.json"
printf 'beta\n' > "$tmp/results/benchmark-report.md"
printf 'ignore me\n' > "$tmp/results/raw/service.log"

out="$tmp/results/manifest.json"
"$root_dir/scripts/build_artifact_manifest.sh" "$tmp/results" "$out" >/dev/null

if ! jq -e '.version == "0.1"' "$out" >/dev/null; then
  echo "manifest missing version" >&2
  exit 1
fi

if ! jq -e '.artifacts | map(.path) | index("summaries/a.json") != null' "$out" >/dev/null; then
  echo "manifest missing summaries/a.json artifact" >&2
  exit 1
fi

if ! jq -e '.artifacts | map(.path) | index("benchmark-report.md") != null' "$out" >/dev/null; then
  echo "manifest missing benchmark-report.md artifact" >&2
  exit 1
fi

if jq -e '.artifacts | map(.path) | index("raw/service.log") != null' "$out" >/dev/null; then
  echo "manifest should exclude .log artifacts" >&2
  exit 1
fi

if "$root_dir/scripts/build_artifact_manifest.sh" "$tmp/does-not-exist" "$out" >/dev/null 2>&1; then
  echo "expected missing results dir to fail" >&2
  exit 1
fi

echo "build_artifact_manifest test passed"
