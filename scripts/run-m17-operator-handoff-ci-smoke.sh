#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--project <path>] [--artifacts-root <path>] [--serve-timeout-ms <ms>] [--max-body-bytes <bytes>]

Runs M17 operator handoff quickstart in CI-safe mode (`--no-matrix`).
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_project=""
artifacts_root=""
serve_timeout_ms=""
max_body_bytes=""

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

quickstart_script="${repo_root}/scripts/run-m17-operator-handoff-quickstart.sh"

if [ ! -f "${quickstart_script}" ]; then
  echo "missing M17 handoff quickstart script: ${quickstart_script}" >&2
  exit 1
fi
if [ ! -x "${quickstart_script}" ]; then
  echo "M17 handoff quickstart script is not executable: ${quickstart_script}" >&2
  exit 1
fi

tmp_artifacts_root=""
if [ -z "${artifacts_root}" ]; then
  tmp_artifacts_root="$(mktemp -d)"
  artifacts_root="${tmp_artifacts_root}"
fi
cleanup() {
  if [ -n "${tmp_artifacts_root}" ]; then
    rm -rf "${tmp_artifacts_root}"
  fi
}
trap cleanup EXIT

quickstart_cmd=(
  "${quickstart_script}"
  --repo-root "${repo_root}"
  --project "${source_project}"
  --artifacts-root "${artifacts_root}"
  --no-matrix
)
if [ -n "${serve_timeout_ms}" ]; then
  quickstart_cmd+=(--serve-timeout-ms "${serve_timeout_ms}")
fi
if [ -n "${max_body_bytes}" ]; then
  quickstart_cmd+=(--max-body-bytes "${max_body_bytes}")
fi

"${quickstart_cmd[@]}"

echo "m17 operator handoff ci smoke passed"
