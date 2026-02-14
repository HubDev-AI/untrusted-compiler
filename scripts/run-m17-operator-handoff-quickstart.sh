#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--project <path>] [--artifacts-root <path>] [--serve-timeout-ms <ms>] [--max-body-bytes <bytes>] [--no-matrix]

Runs the M17 operator handoff quickstart in deterministic order:
  1) handoff readiness verification
  2) operator bootstrap profile (default + max-body + bundle)
  3) strict milestone closure verification
  4) troubleshooting matrix print (unless --no-matrix)
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_project=""
artifacts_root=""
serve_timeout_ms=""
max_body_bytes=""
print_matrix="true"

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
    --no-matrix)
      print_matrix="false"
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

readiness_script="${repo_root}/scripts/check-m17-operator-handoff-readiness.sh"
bootstrap_script="${repo_root}/scripts/run-m17-operator-bootstrap.sh"
closure_script="${repo_root}/scripts/check-milestone-closure.sh"
matrix_script="${repo_root}/scripts/print-m17-operator-troubleshooting-matrix.sh"

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
require_executable "${bootstrap_script}" "M17 bootstrap profile helper"
require_executable "${closure_script}" "milestone closure checker"
require_executable "${matrix_script}" "M17 troubleshooting matrix"

"${readiness_script}" --repo-root "${repo_root}"

bootstrap_cmd=(
  "${bootstrap_script}"
  --repo-root "${repo_root}"
  --project "${source_project}"
  --artifacts-root "${artifacts_root}"
)
if [ -n "${serve_timeout_ms}" ]; then
  bootstrap_cmd+=(--serve-timeout-ms "${serve_timeout_ms}")
fi
if [ -n "${max_body_bytes}" ]; then
  bootstrap_cmd+=(--max-body-bytes "${max_body_bytes}")
fi
"${bootstrap_cmd[@]}"

"${closure_script}" --repo-root "${repo_root}" --fail-on-pending

if [ "${print_matrix}" = "true" ]; then
  "${matrix_script}"
fi

echo "m17 operator handoff quickstart passed"
