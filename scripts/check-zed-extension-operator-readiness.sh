#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/check-zed-extension-operator-readiness.sh [options]

Runs the consolidated operator readiness flow:
  1) release checklist (manifest + grammar pin),
  2) clean-profile install/smoke/rollback flow,
  3) deterministic bundle staging.

Options:
  --skip-release         Skip release checklist step.
  --skip-operator-smoke  Skip operator smoke step.
  --skip-stage           Skip bundle staging step.
  --stage-output <path>  Override stage output root.
  -h, --help             Show this help.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_check="${repo_root}/scripts/check-zed-extension-release.sh"
operator_smoke="${repo_root}/scripts/run-zed-extension-operator-smoke.sh"
stage_bundle="${repo_root}/scripts/stage-zed-extension-bundle.sh"

skip_release=0
skip_operator_smoke=0
skip_stage=0
stage_output=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-release)
      skip_release=1
      shift
      ;;
    --skip-operator-smoke)
      skip_operator_smoke=1
      shift
      ;;
    --skip-stage)
      skip_stage=1
      shift
      ;;
    --stage-output)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      stage_output="$2"
      shift 2
      ;;
    --stage-output=*)
      stage_output="${1#--stage-output=}"
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

if [ ! -x "${release_check}" ]; then
  echo "missing executable script: ${release_check}" >&2
  exit 1
fi
if [ ! -x "${operator_smoke}" ]; then
  echo "missing executable script: ${operator_smoke}" >&2
  exit 1
fi
if [ ! -x "${stage_bundle}" ]; then
  echo "missing executable script: ${stage_bundle}" >&2
  exit 1
fi

if [ "${skip_release}" -eq 0 ]; then
  echo "[zed-operator-readiness] release-check (--skip-smoke)"
  "${release_check}" --skip-smoke
fi

if [ "${skip_operator_smoke}" -eq 0 ]; then
  echo "[zed-operator-readiness] operator-smoke"
  "${operator_smoke}"
fi

if [ "${skip_stage}" -eq 0 ]; then
  echo "[zed-operator-readiness] stage-bundle"
  if [ -n "${stage_output}" ]; then
    "${stage_bundle}" --clean --output-dir "${stage_output}"
    manifest_path="${stage_output}/bundle-manifest.json"
  else
    "${stage_bundle}" --clean
    manifest_path="${repo_root}/build/zed-extension-bundle/bundle-manifest.json"
  fi
  if [ ! -f "${manifest_path}" ]; then
    echo "operator readiness failed: bundle manifest missing at ${manifest_path}" >&2
    exit 1
  fi
fi

echo "[zed-operator-readiness] passed"
