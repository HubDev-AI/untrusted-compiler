#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 --entry <trend-note-entry.md> [--chapter <docs/book/322-...md>]

Imports a rendered trend-note entry into the first trend-note chapter, deduplicated by entry heading.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

entry_path=""
chapter_path="${repo_root}/docs/book/322-m13-first-trend-run-results-note.md"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --entry)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      entry_path="$2"
      shift 2
      ;;
    --entry=*)
      entry_path="${1#--entry=}"
      shift
      ;;
    --chapter)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      chapter_path="$2"
      shift 2
      ;;
    --chapter=*)
      chapter_path="${1#--chapter=}"
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

if [ -z "${entry_path}" ]; then
  echo "missing required --entry argument" >&2
  usage
  exit 2
fi

if [ ! -f "${entry_path}" ]; then
  echo "entry file not found: ${entry_path}" >&2
  exit 2
fi

if [ ! -f "${chapter_path}" ]; then
  echo "chapter file not found: ${chapter_path}" >&2
  exit 2
fi

entry_heading="$(grep -E '^## Trend Entry \(.+\)$' "${entry_path}" | head -n 1 || true)"
if [ -z "${entry_heading}" ]; then
  echo "entry file missing required heading line: ## Trend Entry (<date>)" >&2
  exit 2
fi

if grep -Fq "${entry_heading}" "${chapter_path}"; then
  echo "already imported: ${entry_heading}"
  exit 0
fi

{
  echo
  cat "${entry_path}"
} >> "${chapter_path}"

echo "imported ${entry_heading} into ${chapter_path}"
