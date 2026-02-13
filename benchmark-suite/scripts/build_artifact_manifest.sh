#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <results_dir> <out_manifest.json>" >&2
  exit 2
fi

results_dir="$1"
out_path="$2"

if [ ! -d "$results_dir" ]; then
  echo "results directory not found: ${results_dir}" >&2
  exit 2
fi

if ! command -v shasum >/dev/null 2>&1; then
  echo "shasum is required to build artifact manifest" >&2
  exit 127
fi

tmp_manifest="$(mktemp)"
trap 'rm -f "$tmp_manifest"' EXIT

{
  echo "{"
  echo '  "version": "0.1",'
  echo "  \"generatedAtUtc\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\","
  echo "  \"resultsDir\": \"${results_dir}\","
  echo '  "artifacts": ['

  first="true"
  find "$results_dir" -type f \
    ! -name '*.log' \
    ! -name '.gitkeep' \
    | sort | while IFS= read -r file; do
        rel="${file#$results_dir/}"
        size_bytes="$(wc -c < "$file" | tr -d ' ')"
        sha256="$(shasum -a 256 "$file" | awk '{print $1}')"
        if [ "$first" = "true" ]; then
          first="false"
        else
          echo ","
        fi
        printf '    {"path":"%s","sizeBytes":%s,"sha256":"%s"}' "$rel" "$size_bytes" "$sha256"
      done
  echo
  echo "  ]"
  echo "}"
} > "$tmp_manifest"

mkdir -p "$(dirname "$out_path")"
mv "$tmp_manifest" "$out_path"
trap - EXIT

echo "wrote $out_path"
