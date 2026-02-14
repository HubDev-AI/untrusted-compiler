#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--artifacts-root <path>] [--format <text|json>]

Inspects M17 operator-handoff artifacts and prints deterministic branch summaries.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_root=""
output_format="text"

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
    --format)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_format="$2"
      shift 2
      ;;
    --format=*)
      output_format="${1#--format=}"
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

case "${output_format}" in
  text|json)
    ;;
  *)
    echo "unknown format: ${output_format}" >&2
    usage
    exit 2
    ;;
esac

if [ -z "${artifacts_root}" ]; then
  artifacts_root="${repo_root}/build/operator-handoff-smoke"
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

bundle_checker="${repo_root}/scripts/check-runtime-smoke-bundle.sh"
if [ ! -f "${bundle_checker}" ]; then
  echo "missing runtime smoke bundle checker: ${bundle_checker}" >&2
  exit 1
fi
if [ ! -x "${bundle_checker}" ]; then
  echo "runtime smoke bundle checker is not executable: ${bundle_checker}" >&2
  exit 1
fi

index_path="${artifacts_root}/runtime-smoke-branch-index.json"
"${bundle_checker}" --artifacts-root "${artifacts_root}" --index-path "${index_path}" >/dev/null

branch_names=("default" "max-body")
branches_json='[]'

meta_value() {
  local key="$1"
  local file="$2"
  local value
  value="$(awk -F= -v k="${key}" '$1 == k { print $2; found=1 } END { if (!found) exit 1 }' "${file}" 2>/dev/null || true)"
  if [ -z "${value}" ]; then
    echo ""
    return
  fi
  printf '%s\n' "${value}"
}

for branch in "${branch_names[@]}"; do
  branch_dir="${artifacts_root}/${branch}"
  meta_file="${branch_dir}/run-metadata.txt"
  health_headers="${branch_dir}/health.headers"
  users_headers="${branch_dir}/users.headers"

  if [ ! -f "${meta_file}" ]; then
    echo "missing branch metadata file: ${meta_file}" >&2
    exit 1
  fi
  if [ ! -f "${health_headers}" ]; then
    echo "missing health headers file: ${health_headers}" >&2
    exit 1
  fi
  if [ ! -f "${users_headers}" ]; then
    echo "missing users headers file: ${users_headers}" >&2
    exit 1
  fi

  max_body="$(meta_value "maxBodyBytes" "${meta_file}")"
  run_flags="$(meta_value "runFlags" "${meta_file}")"
  port="$(meta_value "port" "${meta_file}")"
  timeout_ms="$(meta_value "serveTimeoutMs" "${meta_file}")"
  health_status="$(sed -n '1p' "${health_headers}" | tr -d '\r')"
  users_status="$(sed -n '1p' "${users_headers}" | tr -d '\r')"

  branches_json="$(
    jq \
      --arg name "${branch}" \
      --arg maxBodyBytes "${max_body}" \
      --arg runFlags "${run_flags}" \
      --arg port "${port}" \
      --arg serveTimeoutMs "${timeout_ms}" \
      --arg healthStatus "${health_status}" \
      --arg usersStatus "${users_status}" \
      '. + [{
        name: $name,
        maxBodyBytes: $maxBodyBytes,
        runFlags: $runFlags,
        port: $port,
        serveTimeoutMs: $serveTimeoutMs,
        healthStatus: $healthStatus,
        usersStatus: $usersStatus
      }]' \
      <<<"${branches_json}"
  )"
done

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg artifactsRoot "${artifacts_root}" \
    --arg indexPath "${index_path}" \
    --argjson branches "${branches_json}" \
    '{
      version: $version,
      artifactsRoot: $artifactsRoot,
      indexPath: $indexPath,
      branchCount: ($branches | length),
      branches: $branches
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

echo "M17 Operator Handoff Artifact Summary"
echo "artifactsRoot: ${artifacts_root}"
echo "indexPath: ${index_path}"
echo "branchCount: $(jq -r '.branchCount' <<<"${summary_json}")"
echo
jq -r '.branches[] | "branch=\(.name) port=\(.port) timeoutMs=\(.serveTimeoutMs) maxBodyBytes=\(.maxBodyBytes) runFlags=\(.runFlags) healthStatus=\(.healthStatus) usersStatus=\(.usersStatus)"' <<<"${summary_json}"
