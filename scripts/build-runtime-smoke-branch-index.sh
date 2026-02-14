#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--artifacts-root <path>] [--out <path>]

Builds an aggregated runtime-smoke branch index for default and max-body smoke runs.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_root="${root_dir}/build/runtime-smoke"
out_path=""

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
    --out)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_path="$2"
      shift 2
      ;;
    --out=*)
      out_path="${1#--out=}"
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

if [ -z "${out_path}" ]; then
  out_path="${artifacts_root}/runtime-smoke-branch-index.json"
fi

require_file() {
  local branch="$1"
  local path="$2"
  local base="$3"
  if [ ! -f "${path}" ]; then
    echo "missing runtime-smoke ${branch} artifact file: ${base}" >&2
    exit 1
  fi
  if [ ! -s "${path}" ]; then
    echo "runtime-smoke ${branch} artifact file is empty: ${base}" >&2
    exit 1
  fi
}

read_field() {
  local file="$1"
  local key="$2"
  sed -n "s/^${key}=//p" "${file}" | head -n 1
}

branches_json='[]'

for branch in default max-body; do
  branch_dir="${artifacts_root}/${branch}"
  if [ ! -d "${branch_dir}" ]; then
    echo "missing runtime-smoke branch artifacts directory: ${branch}" >&2
    exit 1
  fi

  run_metadata_path="${branch_dir}/run-metadata.txt"
  users_headers_path="${branch_dir}/users.headers"
  users_body_path="${branch_dir}/users.body"

  require_file "${branch}" "${run_metadata_path}" "run-metadata.txt"
  require_file "${branch}" "${users_headers_path}" "users.headers"
  require_file "${branch}" "${users_body_path}" "users.body"

  source_project="$(read_field "${run_metadata_path}" "sourceProject")"
  work_project="$(read_field "${run_metadata_path}" "workProject")"
  port="$(read_field "${run_metadata_path}" "port")"
  oneshot="$(read_field "${run_metadata_path}" "oneshot")"
  serve_timeout_ms="$(read_field "${run_metadata_path}" "serveTimeoutMs")"
  max_body_bytes="$(read_field "${run_metadata_path}" "maxBodyBytes")"
  run_flags="$(read_field "${run_metadata_path}" "runFlags")"

  if [ -z "${source_project}" ] || [ -z "${work_project}" ] || [ -z "${port}" ] || [ -z "${oneshot}" ] || [ -z "${serve_timeout_ms}" ] || [ -z "${max_body_bytes}" ] || [ -z "${run_flags}" ]; then
    echo "runtime-smoke ${branch} metadata is missing required fields" >&2
    exit 1
  fi

  if ! [[ "${max_body_bytes}" =~ ^(unset|[0-9]+)$ ]]; then
    echo "runtime-smoke ${branch} maxBodyBytes has invalid shape: ${max_body_bytes}" >&2
    exit 1
  fi

  users_header_trace_id="$(sed -nE 's/^X-Trace-Id:[[:space:]]*//Ip' "${users_headers_path}" | tr -d '\r' | head -n 1)"
  users_body_trace_id="$(jq -r '.traceId // empty' "${users_body_path}")"
  if [ -z "${users_header_trace_id}" ] || [ -z "${users_body_trace_id}" ]; then
    echo "runtime-smoke ${branch} traceId fields are missing" >&2
    exit 1
  fi
  if [ "${users_header_trace_id}" != "${users_body_trace_id}" ]; then
    echo "runtime-smoke ${branch} users traceId mismatch between header and body" >&2
    exit 1
  fi

  expected_run_flags="--port,--oneshot,--serve-timeout-ms"
  if [ "${branch}" = "default" ]; then
    if [ "${max_body_bytes}" != "unset" ]; then
      echo "runtime-smoke index expects default branch maxBodyBytes=unset" >&2
      exit 1
    fi
  else
    if [ "${max_body_bytes}" = "unset" ]; then
      echo "runtime-smoke index expects max-body branch maxBodyBytes to be numeric" >&2
      exit 1
    fi
    expected_run_flags="${expected_run_flags},--max-body-bytes"
  fi

  if [ "${run_flags}" != "${expected_run_flags}" ]; then
    echo "runtime-smoke ${branch} runFlags shape does not match expected branch profile" >&2
    exit 1
  fi

  branch_json="$(
    jq -n \
      --arg branch "${branch}" \
      --arg dir "${branch_dir}" \
      --arg sourceProject "${source_project}" \
      --arg workProject "${work_project}" \
      --arg port "${port}" \
      --arg oneshot "${oneshot}" \
      --arg serveTimeoutMs "${serve_timeout_ms}" \
      --arg maxBodyBytes "${max_body_bytes}" \
      --arg runFlags "${run_flags}" \
      --arg traceId "${users_body_trace_id}" \
      '{
        branch: $branch,
        dir: $dir,
        sourceProject: $sourceProject,
        workProject: $workProject,
        port: $port,
        oneshot: $oneshot,
        serveTimeoutMs: $serveTimeoutMs,
        maxBodyBytes: $maxBodyBytes,
        runFlags: $runFlags,
        traceId: $traceId
      }'
  )"

  branches_json="$(jq --argjson item "${branch_json}" '. + [$item]' <<<"${branches_json}")"
done

mkdir -p "$(dirname "${out_path}")"
jq -n \
  --arg version "0.1" \
  --arg root "${artifacts_root}" \
  --argjson branchOrder '["default","max-body"]' \
  --argjson branches "${branches_json}" \
  '{
    version: $version,
    root: $root,
    branchOrder: $branchOrder,
    branches: $branches
  }' > "${out_path}"

echo "runtime-smoke branch index generated: ${out_path}"
