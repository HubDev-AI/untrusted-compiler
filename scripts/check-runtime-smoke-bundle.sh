#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--artifacts-root <path>] [--index-path <path>]

Validates runtime-smoke hello-api and LASM db-adapter artifact branches and builds aggregated hello-api branch index.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_root="${repo_root}/build/runtime-smoke"
index_path=""

while [ "$#" -gt 0 ]; do
  case "$1" in
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
    --index-path)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      index_path="$2"
      shift 2
      ;;
    --index-path=*)
      index_path="${1#--index-path=}"
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

if [ ! -d "${artifacts_root}" ]; then
  echo "runtime-smoke artifacts root does not exist: ${artifacts_root}" >&2
  exit 1
fi

if [ -z "${index_path}" ]; then
  index_path="${artifacts_root}/runtime-smoke-branch-index.json"
fi

for branch in default max-body; do
  branch_dir="${artifacts_root}/${branch}"
  if [ ! -d "${branch_dir}" ]; then
    echo "runtime-smoke bundle missing branch directory: ${branch}" >&2
    exit 1
  fi
  "${repo_root}/scripts/check-runtime-smoke-artifacts.sh" --artifacts-dir "${branch_dir}"
done

require_postgres_db_adapter="${LASM_SMOKE_REQUIRE_POSTGRES_DB_ADAPTER:-0}"
db_branches=(lasm-db-records-log lasm-db-sqlite)
if [ "${require_postgres_db_adapter}" = "1" ] || [ "${require_postgres_db_adapter}" = "true" ] || [ "${require_postgres_db_adapter}" = "TRUE" ]; then
  db_branches+=(lasm-db-postgres)
fi

for branch in "${db_branches[@]}"; do
  branch_dir="${artifacts_root}/${branch}"
  if [ ! -d "${branch_dir}" ]; then
    echo "runtime-smoke bundle missing branch directory: ${branch}" >&2
    exit 1
  fi
  "${repo_root}/scripts/check-runtime-smoke-db-adapter-artifacts.sh" --artifacts-dir "${branch_dir}"
done

"${repo_root}/scripts/build-runtime-smoke-branch-index.sh" --artifacts-root "${artifacts_root}" --out "${index_path}"

if ! jq -e '.version == "0.1" and .branchOrder == ["default","max-body"] and (.branches | type == "array" and length == 2)' "${index_path}" >/dev/null; then
  echo "runtime-smoke branch index does not match expected bundle contract" >&2
  exit 1
fi

echo "runtime-smoke bundle check passed"
