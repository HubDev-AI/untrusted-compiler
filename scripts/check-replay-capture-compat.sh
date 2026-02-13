#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/check-replay-capture-contract.sh"
capture_path="${root_dir}/captures/sample-capture.json"
expected_policy_hash=""
expected_compiler_hash=""
expected_runtime_hash=""
allow_policy_mismatch="false"

usage() {
  cat >&2 <<USAGE
usage: $0 --policy-hash <hash> --compiler-hash <hash> --runtime-hash <hash> [--capture <path>] [--allow-policy-mismatch]

Validates replay capture identity-hash compatibility against expected policy/compiler/runtime hashes.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --capture)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --capture" >&2
        usage
        exit 2
      fi
      capture_path="$2"
      shift 2
      ;;
    --capture=*)
      capture_path="${1#--capture=}"
      shift
      ;;
    --policy-hash)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --policy-hash" >&2
        usage
        exit 2
      fi
      expected_policy_hash="$2"
      shift 2
      ;;
    --policy-hash=*)
      expected_policy_hash="${1#--policy-hash=}"
      shift
      ;;
    --compiler-hash)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --compiler-hash" >&2
        usage
        exit 2
      fi
      expected_compiler_hash="$2"
      shift 2
      ;;
    --compiler-hash=*)
      expected_compiler_hash="${1#--compiler-hash=}"
      shift
      ;;
    --runtime-hash)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-hash" >&2
        usage
        exit 2
      fi
      expected_runtime_hash="$2"
      shift 2
      ;;
    --runtime-hash=*)
      expected_runtime_hash="${1#--runtime-hash=}"
      shift
      ;;
    --allow-policy-mismatch)
      allow_policy_mismatch="true"
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

if [[ -z "${expected_policy_hash}" || -z "${expected_compiler_hash}" || -z "${expected_runtime_hash}" ]]; then
  echo "policy/compiler/runtime hash arguments are required" >&2
  usage
  exit 2
fi

"${contract_script}" --capture "${capture_path}" >/dev/null

capture_policy_hash="$(jq -r '.policyHash // ""' "${capture_path}")"
capture_compiler_hash="$(jq -r '.compilerHash // ""' "${capture_path}")"
capture_runtime_hash="$(jq -r '.runtimeHash // ""' "${capture_path}")"

if [[ "${capture_compiler_hash}" != "${expected_compiler_hash}" ]]; then
  echo "replay compatibility failed: compilerHash mismatch (${capture_compiler_hash} != ${expected_compiler_hash})" >&2
  exit 1
fi

if [[ "${capture_runtime_hash}" != "${expected_runtime_hash}" ]]; then
  echo "replay compatibility failed: runtimeHash mismatch (${capture_runtime_hash} != ${expected_runtime_hash})" >&2
  exit 1
fi

if [[ "${capture_policy_hash}" != "${expected_policy_hash}" ]]; then
  if [[ "${allow_policy_mismatch}" == "true" ]]; then
    echo "warning: policyHash mismatch allowed (${capture_policy_hash} != ${expected_policy_hash})" >&2
  else
    echo "replay compatibility failed: policyHash mismatch (${capture_policy_hash} != ${expected_policy_hash})" >&2
    exit 1
  fi
fi

echo "replay capture compatibility check passed"
