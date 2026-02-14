#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--artifacts-root <path>] [--output-dir <path>]

Builds an M17 operator release packet:
  - readiness summary JSON
  - closure gate snapshot JSON
  - artifact hash manifest
  - packet metadata JSON
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_root=""
output_dir=""

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
    --output-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_dir="$2"
      shift 2
      ;;
    --output-dir=*)
      output_dir="${1#--output-dir=}"
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

if [ -z "${artifacts_root}" ]; then
  artifacts_root="${repo_root}/build/operator-handoff-smoke"
fi
if [ -z "${output_dir}" ]; then
  output_dir="${repo_root}/build/operator-release-packet"
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

summary_script="${repo_root}/scripts/summarize-m17-operator-handoff-readiness.sh"
closure_script="${repo_root}/scripts/check-milestone-closure.sh"

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

require_executable "${summary_script}" "M17 readiness summary script"
require_executable "${closure_script}" "milestone closure checker"

if [ ! -d "${artifacts_root}" ]; then
  echo "missing artifacts root: ${artifacts_root}" >&2
  exit 1
fi

mkdir -p "${output_dir}"

summary_json="$("${summary_script}" --repo-root "${repo_root}" --artifacts-root "${artifacts_root}" --format json)"
closure_json="$("${closure_script}" --repo-root "${repo_root}" --format json --fail-on-pending)"

summary_path="${output_dir}/readiness-summary.json"
closure_path="${output_dir}/closure-gates.json"
manifest_path="${output_dir}/artifact-manifest.txt"
packet_path="${output_dir}/release-packet.json"

printf '%s\n' "${summary_json}" > "${summary_path}"
printf '%s\n' "${closure_json}" > "${closure_path}"

hash_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file}" | awk '{print $1}'
    return
  fi
  echo "missing required command: sha256sum or shasum" >&2
  exit 1
}

: > "${manifest_path}"
while IFS= read -r rel_path; do
  [ -z "${rel_path}" ] && continue
  full_path="${artifacts_root}/${rel_path}"
  checksum="$(hash_file "${full_path}")"
  printf '%s  %s\n' "${checksum}" "${rel_path}" >> "${manifest_path}"
done < <(
  cd "${artifacts_root}" && find . -type f | sed 's|^\./||' | sort
)

artifact_count="$(wc -l < "${manifest_path}" | tr -d ' ')"
summary_overall="$(jq -r '.closure.overall' <<<"${summary_json}")"
closure_overall="$(jq -r '.overall' <<<"${closure_json}")"

jq -n \
  --arg version "0.1" \
  --arg artifactsRoot "${artifacts_root}" \
  --arg outputDir "${output_dir}" \
  --arg summaryFile "${summary_path}" \
  --arg closureFile "${closure_path}" \
  --arg manifestFile "${manifest_path}" \
  --arg summaryOverall "${summary_overall}" \
  --arg closureOverall "${closure_overall}" \
  --argjson artifactCount "${artifact_count}" \
  '{
    version: $version,
    artifactsRoot: $artifactsRoot,
    outputDir: $outputDir,
    summaryFile: $summaryFile,
    closureFile: $closureFile,
    manifestFile: $manifestFile,
    summaryOverall: $summaryOverall,
    closureOverall: $closureOverall,
    artifactCount: $artifactCount
  }' > "${packet_path}"

echo "m17 operator release packet built: ${output_dir}"
