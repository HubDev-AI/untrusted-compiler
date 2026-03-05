#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>]

Validates M17 final operator handoff playbook chapter contracts.
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

quickstart_script="${repo_root}/scripts/run-m17-operator-handoff-quickstart.sh"
ci_smoke_script="${repo_root}/scripts/run-m17-operator-handoff-ci-smoke.sh"
summary_script="${repo_root}/scripts/summarize-m17-operator-handoff-readiness.sh"
inspector_script="${repo_root}/scripts/inspect-m17-operator-handoff-artifacts.sh"
release_packet_script="${repo_root}/scripts/build-m17-operator-release-packet.sh"
closure_script="${repo_root}/scripts/check-milestone-closure.sh"
playbook_chapter="${repo_root}/docs/book/473-m17-operator-handoff-final-playbook.md"
zed_readiness_script="${repo_root}/scripts/check-zed-extension-operator-readiness.sh"

require_executable "${quickstart_script}" "M17 handoff quickstart script"
require_executable "${ci_smoke_script}" "M17 handoff CI smoke wrapper"
require_executable "${summary_script}" "M17 readiness summary script"
require_executable "${inspector_script}" "M17 artifact inspector script"
require_executable "${release_packet_script}" "M17 release packet builder"
require_executable "${closure_script}" "milestone closure checker"
require_executable "${zed_readiness_script}" "zed extension operator readiness checker"
require_file "${playbook_chapter}" "M17 final handoff playbook chapter"

require_token "${playbook_chapter}" '## Bundle A: Local Validation Flow' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/check-m17-operator-handoff-readiness.sh --repo-root .' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/run-m17-operator-handoff-quickstart.sh --repo-root . --project examples/hello-api --artifacts-root build/runtime-smoke' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text' "playbook chapter"

require_token "${playbook_chapter}" '## Bundle B: CI Smoke and Artifact Inspection Flow' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/run-m17-operator-handoff-ci-smoke.sh --repo-root . --artifacts-root build/operator-handoff-smoke' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/inspect-m17-operator-handoff-artifacts.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/check-milestone-closure.sh --repo-root . --fail-on-pending' "playbook chapter"

require_token "${playbook_chapter}" '## Bundle C: Release Packet Assembly Flow' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/build-m17-operator-release-packet.sh --repo-root . --artifacts-root build/operator-handoff-smoke --output-dir build/operator-release-packet' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format json' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/check-milestone-closure.sh --repo-root . --format json --fail-on-pending' "playbook chapter"

require_token "${playbook_chapter}" 'build/operator-release-packet/release-packet.json' "playbook chapter"
require_token "${playbook_chapter}" 'build/operator-release-packet/artifact-manifest.txt' "playbook chapter"
require_token "${playbook_chapter}" '## Bundle D: Zed Extension Operator Readiness' "playbook chapter"
require_token "${playbook_chapter}" 'scripts/check-zed-extension-operator-readiness.sh --stage-output build/zed-extension-release-operator' "playbook chapter"
require_token "${playbook_chapter}" 'build/zed-extension-release-operator/bundle-manifest.json' "playbook chapter"

echo "m17 operator handoff playbook check passed"
