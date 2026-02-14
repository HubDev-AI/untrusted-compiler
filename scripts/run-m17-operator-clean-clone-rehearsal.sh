#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--clone-root <path>] [--output-dir <path>] [--skip-clone]

Runs a deterministic M17 operator handoff rehearsal and captures friction evidence.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
clone_root=""
output_dir=""
skip_clone="false"

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
    --clone-root)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      clone_root="$2"
      shift 2
      ;;
    --clone-root=*)
      clone_root="${1#--clone-root=}"
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
    --skip-clone)
      skip_clone="true"
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

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "missing required command: ${name}" >&2
    exit 1
  fi
}

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

if [ ! -d "${repo_root}" ]; then
  echo "missing repo root: ${repo_root}" >&2
  exit 1
fi

require_cmd jq
if [ "${skip_clone}" = "false" ]; then
  require_cmd git
fi

if [ -z "${output_dir}" ]; then
  output_dir="${repo_root}/build/operator-clean-clone-rehearsal"
fi
mkdir -p "${output_dir}"

logs_dir="${output_dir}/logs"
rm -rf "${logs_dir}"
mkdir -p "${logs_dir}"

cleanup_clone_dir=""
work_repo="${repo_root}"
clone_mode="in-place"
if [ "${skip_clone}" = "false" ]; then
  clone_mode="clean-clone"
  if [ -z "${clone_root}" ]; then
    clone_root="$(mktemp -d)"
    cleanup_clone_dir="${clone_root}"
  fi
  mkdir -p "${clone_root}"
  work_repo="${clone_root}/untrusted-clean-clone"
  rm -rf "${work_repo}"
  git clone --quiet "${repo_root}" "${work_repo}"
fi

cleanup() {
  if [ -n "${cleanup_clone_dir}" ]; then
    rm -rf "${cleanup_clone_dir}"
  fi
}
trap cleanup EXIT

local_artifacts_root="${work_repo}/build/runtime-smoke"
ci_artifacts_root="${work_repo}/build/operator-handoff-smoke"
release_packet_dir="${work_repo}/build/operator-release-packet"

playbook_checker="${work_repo}/scripts/check-m17-operator-handoff-playbook.sh"
quickstart_runner="${work_repo}/scripts/run-m17-operator-handoff-quickstart.sh"
ci_smoke_runner="${work_repo}/scripts/run-m17-operator-handoff-ci-smoke.sh"
summary_runner="${work_repo}/scripts/summarize-m17-operator-handoff-readiness.sh"
release_packet_builder="${work_repo}/scripts/build-m17-operator-release-packet.sh"
closure_checker="${work_repo}/scripts/check-milestone-closure.sh"

require_executable "${playbook_checker}" "M17 playbook checker"
require_executable "${quickstart_runner}" "M17 quickstart runner"
require_executable "${ci_smoke_runner}" "M17 CI smoke runner"
require_executable "${summary_runner}" "M17 readiness summary runner"
require_executable "${release_packet_builder}" "M17 release packet builder"
require_executable "${closure_checker}" "milestone closure checker"

steps_json='[]'
friction_json='[]'

record_step() {
  local id="$1"
  local description="$2"
  local status="$3"
  local duration_ms="$4"
  local log_path="$5"
  steps_json="$(
    jq \
      --arg id "${id}" \
      --arg description "${description}" \
      --arg status "${status}" \
      --argjson durationMs "${duration_ms}" \
      --arg logPath "${log_path}" \
      '. + [{id: $id, description: $description, status: $status, durationMs: $durationMs, logPath: $logPath}]' \
      <<<"${steps_json}"
  )"
}

record_friction() {
  local step_id="$1"
  local message="$2"
  friction_json="$(
    jq \
      --arg stepId "${step_id}" \
      --arg message "${message}" \
      '. + [{stepId: $stepId, message: $message}]' \
      <<<"${friction_json}"
  )"
}

run_step() {
  local id="$1"
  local description="$2"
  shift 2
  local log_path="${logs_dir}/${id}.log"
  local start_ts="$(date +%s)"

  if "$@" >"${log_path}" 2>&1; then
    local end_ts="$(date +%s)"
    local duration_ms=$(( (end_ts - start_ts) * 1000 ))
    record_step "${id}" "${description}" "PASS" "${duration_ms}" "${log_path}"
    return 0
  fi

  local end_ts="$(date +%s)"
  local duration_ms=$(( (end_ts - start_ts) * 1000 ))
  record_step "${id}" "${description}" "FAIL" "${duration_ms}" "${log_path}"

  local first_line
  first_line="$(sed -n '1,120p' "${log_path}" | awk 'NF { print; exit }')"
  if [ -z "${first_line}" ]; then
    first_line="step failed without diagnostic output"
  fi
  record_friction "${id}" "${first_line}"
  return 1
}

failed_step=""
if ! run_step "playbook" "Validate final handoff playbook contracts" \
  "${playbook_checker}" --repo-root "${work_repo}"; then
  failed_step="playbook"
elif ! run_step "quickstart" "Run local quickstart flow in clean-clone workspace" \
  "${quickstart_runner}" --repo-root "${work_repo}" --project "${work_repo}/examples/hello-api" --artifacts-root "${local_artifacts_root}" --no-matrix; then
  failed_step="quickstart"
elif ! run_step "ci-smoke" "Run CI smoke wrapper and produce operator artifact bundle" \
  "${ci_smoke_runner}" --repo-root "${work_repo}" --project "${work_repo}/examples/hello-api" --artifacts-root "${ci_artifacts_root}"; then
  failed_step="ci-smoke"
elif ! run_step "release-packet" "Build operator release packet from CI smoke artifacts" \
  "${release_packet_builder}" --repo-root "${work_repo}" --artifacts-root "${ci_artifacts_root}" --output-dir "${release_packet_dir}"; then
  failed_step="release-packet"
elif ! run_step "summary" "Capture JSON readiness summary snapshot" \
  "${summary_runner}" --repo-root "${work_repo}" --artifacts-root "${ci_artifacts_root}" --format json; then
  failed_step="summary"
elif ! run_step "closure" "Run strict closure audit snapshot" \
  "${closure_checker}" --repo-root "${work_repo}" --format json --fail-on-pending; then
  failed_step="closure"
fi

overall="PASS"
if [ -n "${failed_step}" ]; then
  overall="FAIL"
fi

report_json="$(
  jq -n \
    --arg version "0.1" \
    --arg overall "${overall}" \
    --arg cloneMode "${clone_mode}" \
    --arg repoRoot "${repo_root}" \
    --arg workRepo "${work_repo}" \
    --arg outputDir "${output_dir}" \
    --arg failedStep "${failed_step:-none}" \
    --argjson steps "${steps_json}" \
    --argjson friction "${friction_json}" \
    '{
      version: $version,
      overall: $overall,
      cloneMode: $cloneMode,
      repoRoot: $repoRoot,
      workRepo: $workRepo,
      outputDir: $outputDir,
      failedStep: $failedStep,
      steps: $steps,
      friction: $friction
    }'
)"

report_path="${output_dir}/rehearsal-report.json"
printf '%s\n' "${report_json}" > "${report_path}"

echo "M17 Operator Clean-Clone Rehearsal"
echo "overall: ${overall}"
echo "cloneMode: ${clone_mode}"
echo "report: ${report_path}"

if [ "${overall}" != "PASS" ]; then
  exit 1
fi

exit 0
