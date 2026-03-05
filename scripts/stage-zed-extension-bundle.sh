#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/stage-zed-extension-bundle.sh [--output-dir <path>] [--clean]

Stages a deterministic local bundle for zed-extension and emits a hash manifest.

Options:
  --output-dir <path>  Output root (default: build/zed-extension-bundle)
  --clean              Remove existing staged bundle before staging.
  -h, --help           Show this help.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${repo_root}/zed-extension"
manifest_toml="${source_dir}/extension.toml"
output_root="${repo_root}/build/zed-extension-bundle"
clean=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_root="$2"
      shift 2
      ;;
    --output-dir=*)
      output_root="${1#--output-dir=}"
      shift
      ;;
    --clean)
      clean=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [ ! -f "${manifest_toml}" ]; then
  echo "missing manifest: ${manifest_toml}" >&2
  exit 1
fi

extension_id="$(awk -F'"' '/^id[[:space:]]*=[[:space:]]*"/ { print $2; exit }' "${manifest_toml}")"
extension_version="$(awk -F'"' '/^version[[:space:]]*=[[:space:]]*"/ { print $2; exit }' "${manifest_toml}")"

if [ -z "${extension_id}" ] || [ -z "${extension_version}" ]; then
  echo "failed to parse id/version from ${manifest_toml}" >&2
  exit 1
fi

bundle_dir="${output_root}/${extension_id}-${extension_version}"
manifest_json="${output_root}/bundle-manifest.json"

if [ "${clean}" -eq 1 ] && [ -d "${bundle_dir}" ]; then
  rm -rf "${bundle_dir}"
fi

mkdir -p "${bundle_dir}"
cp -R "${source_dir}/." "${bundle_dir}/"
find "${bundle_dir}" -name ".DS_Store" -type f -delete

hash_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  else
    shasum -a 256 "${file}" | awk '{print $1}'
  fi
}

tmp_files="$(mktemp)"
(
  cd "${bundle_dir}"
  find . -type f | sed 's#^\./##' | sort
) > "${tmp_files}"

{
  echo "{"
  echo "  \"id\": \"${extension_id}\","
  echo "  \"version\": \"${extension_version}\","
  echo "  \"bundleDir\": \"${bundle_dir}\","
  echo "  \"files\": ["
  first=1
  while IFS= read -r rel_path; do
    [ -n "${rel_path}" ] || continue
    abs_path="${bundle_dir}/${rel_path}"
    size="$(wc -c < "${abs_path}" | tr -d ' ')"
    sha256="$(hash_file "${abs_path}")"
    if [ "${first}" -eq 0 ]; then
      echo "    ,"
    fi
    first=0
    echo "    {"
    echo "      \"path\": \"${rel_path}\","
    echo "      \"size\": ${size},"
    echo "      \"sha256\": \"${sha256}\""
    echo "    }"
  done < "${tmp_files}"
  echo "  ]"
  echo "}"
} > "${manifest_json}"

rm -f "${tmp_files}"

echo "staged zed extension bundle: ${bundle_dir}"
echo "wrote bundle manifest: ${manifest_json}"
