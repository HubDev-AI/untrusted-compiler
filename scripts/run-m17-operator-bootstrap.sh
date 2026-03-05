#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--project <path>] [--artifacts-root <path>] [--serve-timeout-ms <ms>] [--max-body-bytes <bytes>]

Runs the M17 operator bootstrap profile:
  1) default runtime smoke
  2) max-body runtime smoke
  3) LASM DB runtime smoke (records-log)
  4) LASM DB runtime smoke (sqlite)
  5) runtime-smoke bundle validation/index generation
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_project=""
artifacts_root=""
serve_timeout_ms="12000"
max_body_bytes="2048"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo-root)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      repo_root="$2"
      shift 2
      ;;
    --repo-root=*)
      repo_root="${1#--repo-root=}"
      shift
      ;;
    --project)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      source_project="$2"
      shift 2
      ;;
    --project=*)
      source_project="${1#--project=}"
      shift
      ;;
    --artifacts-root)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      artifacts_root="$2"
      shift 2
      ;;
    --artifacts-root=*)
      artifacts_root="${1#--artifacts-root=}"
      shift
      ;;
    --serve-timeout-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      serve_timeout_ms="$2"
      shift 2
      ;;
    --serve-timeout-ms=*)
      serve_timeout_ms="${1#--serve-timeout-ms=}"
      shift
      ;;
    --max-body-bytes)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      max_body_bytes="$2"
      shift 2
      ;;
    --max-body-bytes=*)
      max_body_bytes="${1#--max-body-bytes=}"
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

if [ -z "${source_project}" ]; then
  source_project="${repo_root}/examples/hello-api"
fi
if [ -z "${artifacts_root}" ]; then
  artifacts_root="${repo_root}/build/runtime-smoke"
fi

runtime_smoke_script="${repo_root}/scripts/smoke-sec4-run-hello-api.sh"
db_runtime_smoke_script="${repo_root}/scripts/smoke-sec4-run-lasm-db-adapter.sh"
bundle_checker_script="${repo_root}/scripts/check-runtime-smoke-bundle.sh"

require_executable() {
  local path="$1"
  local label="$2"
  if [ ! -f "${path}" ]; then
    echo "missing ${label}: ${path}" >&2
    exit 1
  fi
  if [ ! -x "${path}" ]; then
    echo "${label} is not executable: ${path}" >&2
    exit 1
  fi
}

if [ ! -d "${source_project}" ]; then
  echo "missing project directory: ${source_project}" >&2
  exit 1
fi

if ! [[ "${serve_timeout_ms}" =~ ^[0-9]+$ ]] || [ "${serve_timeout_ms}" -le 0 ]; then
  echo "invalid --serve-timeout-ms value (expected positive integer): ${serve_timeout_ms}" >&2
  exit 1
fi

if ! [[ "${max_body_bytes}" =~ ^[0-9]+$ ]] || [ "${max_body_bytes}" -le 0 ]; then
  echo "invalid --max-body-bytes value (expected positive integer): ${max_body_bytes}" >&2
  exit 1
fi

require_executable "${runtime_smoke_script}" "runtime smoke script"
require_executable "${db_runtime_smoke_script}" "db runtime smoke script"
require_executable "${bundle_checker_script}" "runtime smoke bundle checker"

mkdir -p "${artifacts_root}"

default_artifacts_dir="${artifacts_root}/default"
max_body_artifacts_dir="${artifacts_root}/max-body"
db_records_log_artifacts_dir="${artifacts_root}/lasm-db-records-log"
db_sqlite_artifacts_dir="${artifacts_root}/lasm-db-sqlite"
branch_index_path="${artifacts_root}/runtime-smoke-branch-index.json"

"${runtime_smoke_script}" \
  --project "${source_project}" \
  --serve-timeout-ms "${serve_timeout_ms}" \
  --artifacts-dir "${default_artifacts_dir}"

"${runtime_smoke_script}" \
  --project "${source_project}" \
  --serve-timeout-ms "${serve_timeout_ms}" \
  --max-body-bytes "${max_body_bytes}" \
  --artifacts-dir "${max_body_artifacts_dir}"

"${db_runtime_smoke_script}" \
  --db-adapter records-log \
  --serve-timeout-ms "${serve_timeout_ms}" \
  --artifacts-dir "${db_records_log_artifacts_dir}"

"${db_runtime_smoke_script}" \
  --db-adapter sqlite \
  --serve-timeout-ms "${serve_timeout_ms}" \
  --artifacts-dir "${db_sqlite_artifacts_dir}"

"${bundle_checker_script}" \
  --artifacts-root "${artifacts_root}" \
  --index-path "${branch_index_path}"

echo "m17 operator bootstrap profile passed"
