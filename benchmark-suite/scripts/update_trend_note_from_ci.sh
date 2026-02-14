#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo owner/repo] [--out-dir <path>] [--chapter <path>] [--entry <trend-note-entry.md>] [--local-matrix <compare-matrix.json>] [--prefer-local] [--dry-run]

Fetches latest benchmark trend artifact (unless --entry is provided), with local compare-matrix fallback, and imports trend-note-entry.md into chapter 322.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

repo_slug="HubDev-AI/untrusted-compiler"
out_dir="${repo_root}/benchmark-suite/results/trend-download"
chapter_path="${repo_root}/docs/book/322-m13-first-trend-run-results-note.md"
entry_path=""
local_matrix_path="${repo_root}/benchmark-suite/results/summaries/compare-matrix.json"
prefer_local="false"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      repo_slug="$2"
      shift 2
      ;;
    --repo=*)
      repo_slug="${1#--repo=}"
      shift
      ;;
    --out-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_dir="$2"
      shift 2
      ;;
    --out-dir=*)
      out_dir="${1#--out-dir=}"
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
    --local-matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      local_matrix_path="$2"
      shift 2
      ;;
    --local-matrix=*)
      local_matrix_path="${1#--local-matrix=}"
      shift
      ;;
    --prefer-local)
      prefer_local="true"
      shift
      ;;
    --dry-run)
      dry_run="true"
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

fetch_cmd=("${script_dir}/fetch_trend_artifact.sh" "--repo" "${repo_slug}" "--out-dir" "${out_dir}")
import_cmd=("${script_dir}/import_trend_note_entry.sh" "--chapter" "${chapter_path}" "--replace-existing")
render_cmd=("${script_dir}/render_trend_note_entry.sh" "${local_matrix_path}")

if [ "${dry_run}" = "true" ]; then
  if [ -n "${entry_path}" ]; then
    echo "run: ${import_cmd[*]} --entry ${entry_path}"
  elif [ "${prefer_local}" = "true" ]; then
    echo "run: ${render_cmd[*]} --out <local trend-note-entry.md>"
    echo "run: ${import_cmd[*]} --entry <local trend-note-entry.md>"
  else
    echo "run: ${fetch_cmd[*]}"
    echo "run: ${import_cmd[*]} --entry <downloaded trend-note-entry.md>"
  fi
  exit 0
fi

cleanup_entry_path=""
cleanup() {
  if [ -n "${cleanup_entry_path}" ] && [ -f "${cleanup_entry_path}" ]; then
    rm -f "${cleanup_entry_path}"
  fi
}
trap cleanup EXIT

render_local_entry() {
  if [ ! -f "${local_matrix_path}" ]; then
    echo "local compare-matrix fallback is missing: ${local_matrix_path}" >&2
    exit 1
  fi
  cleanup_entry_path="$(mktemp)"
  "${script_dir}/render_trend_note_entry.sh" "${local_matrix_path}" --out "${cleanup_entry_path}" >/dev/null
  entry_path="${cleanup_entry_path}"
}

if [ -z "${entry_path}" ]; then
  if [ "${prefer_local}" = "true" ]; then
    render_local_entry
  else
    if "${fetch_cmd[@]}"; then
      entry_path="$(find "${out_dir}" -type f -name 'trend-note-entry.md' | sort | tail -n 1)"
    else
      echo "warning: trend artifact fetch failed, falling back to local compare-matrix render" >&2
      render_local_entry
    fi
    if [ -z "${entry_path}" ]; then
      echo "trend-note-entry.md not found under ${out_dir}; falling back to local compare-matrix render" >&2
      render_local_entry
    fi
  fi
fi

"${import_cmd[@]}" --entry "${entry_path}"
echo "updated trend note chapter: ${chapter_path}"
