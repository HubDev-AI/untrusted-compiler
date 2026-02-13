#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSION_TOML="${ROOT_DIR}/zed-extension/extension.toml"
PLACEHOLDER="TODO_SET_COMMIT_SHA"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --extension)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --extension" >&2
        exit 2
      fi
      EXTENSION_TOML="$2"
      shift 2
      ;;
    --extension=*)
      EXTENSION_TOML="${1#--extension=}"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "${EXTENSION_TOML}" ]]; then
  echo "error: missing ${EXTENSION_TOML}" >&2
  exit 1
fi

GRAMMAR_REV="$(awk -F'"' '/^[[:space:]]*rev[[:space:]]*=[[:space:]]*"/ { print $2; exit }' "${EXTENSION_TOML}")"

if [[ -z "${GRAMMAR_REV}" ]]; then
  echo "error: zed grammar revision is missing in ${EXTENSION_TOML}" >&2
  exit 1
fi

if [[ "${GRAMMAR_REV}" == "${PLACEHOLDER}" ]]; then
  echo "error: zed grammar revision is still a placeholder (${PLACEHOLDER})" >&2
  exit 1
fi

if [[ ! "${GRAMMAR_REV}" =~ ^[0-9a-fA-F]{7,40}$ ]]; then
  echo "error: zed grammar revision is not a commit-like SHA: ${GRAMMAR_REV}" >&2
  exit 1
fi

echo "ok: zed grammar revision appears pinned (${GRAMMAR_REV})"
