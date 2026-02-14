#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>]

Validates M17 operator handoff readiness contracts (scripts, docs, workflows).
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

require_file() {
  local path="$1"
  local label="$2"
  if [ ! -f "${path}" ]; then
    echo "missing ${label}: ${path}" >&2
    exit 1
  fi
}

require_executable() {
  local path="$1"
  local label="$2"
  require_file "${path}" "${label}"
  if [ ! -x "${path}" ]; then
    echo "${label} is not executable: ${path}" >&2
    exit 1
  fi
}

require_token() {
  local file="$1"
  local token="$2"
  local label="$3"
  if ! rg -Fq -- "${token}" "${file}"; then
    echo "missing ${label} token: ${token}" >&2
    exit 1
  fi
}

runtime_smoke_script="${repo_root}/scripts/smoke-sec4-run-hello-api.sh"
bundle_checker_script="${repo_root}/scripts/check-runtime-smoke-bundle.sh"
closure_checker_script="${repo_root}/scripts/check-milestone-closure.sh"
trend_updater_script="${repo_root}/benchmark-suite/scripts/update_trend_note_from_ci.sh"
runtime_smoke_workflow="${repo_root}/.github/workflows/runtime-smoke.yml"
naming_lock_workflow="${repo_root}/.github/workflows/naming-lock.yml"
handoff_chapter="${repo_root}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md"

require_executable "${runtime_smoke_script}" "runtime smoke script"
require_executable "${bundle_checker_script}" "runtime smoke bundle checker"
require_executable "${closure_checker_script}" "milestone closure checker"
require_executable "${trend_updater_script}" "trend-note updater"
require_file "${runtime_smoke_workflow}" "runtime-smoke workflow"
require_file "${naming_lock_workflow}" "naming-lock workflow"
require_file "${handoff_chapter}" "M17 operator handoff checklist chapter"

require_token "${runtime_smoke_workflow}" 'scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default' "runtime-smoke workflow"
require_token "${runtime_smoke_workflow}" 'scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body' "runtime-smoke workflow"
require_token "${runtime_smoke_workflow}" 'scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json' "runtime-smoke workflow"

require_token "${naming_lock_workflow}" 'scripts/test-runtime-smoke-workflow-contract.sh' "naming-lock workflow"
require_token "${naming_lock_workflow}" 'scripts/test-runtime-smoke-workflow-contract-guard.sh' "naming-lock workflow"
require_token "${naming_lock_workflow}" 'scripts/test-check-runtime-smoke-bundle.sh' "naming-lock workflow"
require_token "${naming_lock_workflow}" 'scripts/test-check-m17-operator-handoff-readiness.sh' "naming-lock workflow"

require_token "${handoff_chapter}" 'scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default' "handoff chapter"
require_token "${handoff_chapter}" 'scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body' "handoff chapter"
require_token "${handoff_chapter}" 'scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json' "handoff chapter"
require_token "${handoff_chapter}" 'scripts/check-milestone-closure.sh --fail-on-pending' "handoff chapter"
require_token "${handoff_chapter}" 'benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local' "handoff chapter"

echo "m17 operator handoff readiness check passed"
