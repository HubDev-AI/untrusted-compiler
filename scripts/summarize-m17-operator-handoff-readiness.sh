#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--artifacts-root <path>] [--serve-timeout-ms <ms>] [--max-body-bytes <bytes>] [--format <text|json>]

Runs and summarizes M17 operator handoff readiness:
  1) handoff readiness verifier
  2) handoff CI smoke wrapper
  3) artifact inspector
  4) strict closure audit
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifacts_root=""
serve_timeout_ms=""
max_body_bytes=""
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

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

if [ -z "${artifacts_root}" ]; then
  artifacts_root="${repo_root}/build/operator-handoff-smoke"
fi

readiness_script="${repo_root}/scripts/check-m17-operator-handoff-readiness.sh"
ci_smoke_script="${repo_root}/scripts/run-m17-operator-handoff-ci-smoke.sh"
inspector_script="${repo_root}/scripts/inspect-m17-operator-handoff-artifacts.sh"
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

require_executable "${readiness_script}" "M17 readiness verifier"
require_executable "${ci_smoke_script}" "M17 CI smoke wrapper"
require_executable "${inspector_script}" "M17 artifact inspector"
require_executable "${closure_script}" "milestone closure checker"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

readiness_log="${tmp_dir}/readiness.log"
if ! "${readiness_script}" --repo-root "${repo_root}" >"${readiness_log}" 2>&1; then
  cat "${readiness_log}" >&2
  exit 1
fi

ci_smoke_cmd=(
  "${ci_smoke_script}"
  --repo-root "${repo_root}"
  --artifacts-root "${artifacts_root}"
)
if [ -n "${serve_timeout_ms}" ]; then
  ci_smoke_cmd+=(--serve-timeout-ms "${serve_timeout_ms}")
fi
if [ -n "${max_body_bytes}" ]; then
  ci_smoke_cmd+=(--max-body-bytes "${max_body_bytes}")
fi

ci_smoke_log="${tmp_dir}/ci-smoke.log"
if ! "${ci_smoke_cmd[@]}" >"${ci_smoke_log}" 2>&1; then
  cat "${ci_smoke_log}" >&2
  exit 1
fi

inspector_json="$("${inspector_script}" --repo-root "${repo_root}" --artifacts-root "${artifacts_root}" --format json)"
closure_json="$("${closure_script}" --repo-root "${repo_root}" --format json --fail-on-pending)"

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg artifactsRoot "${artifacts_root}" \
    --argjson inspector "${inspector_json}" \
    --argjson closure "${closure_json}" \
    '{
      version: $version,
      artifactsRoot: $artifactsRoot,
      steps: {
        readiness: "PASS",
        ciSmoke: "PASS",
        artifactInspector: "PASS",
        closure: "PASS"
      },
      inspector: {
        branchCount: $inspector.branchCount,
        branches: $inspector.branches
      },
      closure: {
        overall: $closure.overall,
        pendingCount: $closure.pendingCount
      }
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

echo "M17 Operator Handoff Readiness Summary"
echo "artifactsRoot: ${artifacts_root}"
echo "steps: readiness=PASS ciSmoke=PASS artifactInspector=PASS closure=PASS"
echo "branchCount: $(jq -r '.inspector.branchCount' <<<"${summary_json}")"
echo "closureOverall: $(jq -r '.closure.overall' <<<"${summary_json}")"
