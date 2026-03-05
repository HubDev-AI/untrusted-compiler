#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

output_format="text"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --json)
      output_format="json"
      shift
      ;;
    -h|--help)
      cat <<'USAGE'
usage: scripts/print-test-inventory.sh [--json]

Prints a lightweight inventory of Rust and shell test coverage.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

rust_test_files="$(rg --files compiler/sec4-core/tests compiler/sec4-cli/tests | rg '\.rs$' | wc -l | tr -d ' ')"
rust_test_count="$(rg -n '^\s*#\[test\]' compiler/sec4-core/tests compiler/sec4-cli/tests | wc -l | tr -d ' ')"

shell_total=0
shell_impl=0
shell_milestone=0
shell_other=0

impl_pattern='(sec4|runtime|replay|zed|json|audit|smoke-sec4-run|check-sec4|verify-release|check-no-local-path-leaks|check-runtime-smoke|build-runtime-smoke-branch-index)'
milestone_pattern='(workflow-contract|contract-guard|operator-handoff|closure|kickoff|priority-matrix|transition-handoff|select-m[0-9]+|plan-m[0-9]+|generate-m[0-9]+|run-m[0-9]+|build-m[0-9]+|check-milestone-closure|roadmap-closure|release-contract|alpha-release|benchmark-cross-impl|benchmark-trend|refresh-closure)'

for script_path in scripts/test-*.sh; do
  [ -e "${script_path}" ] || continue
  shell_total=$((shell_total + 1))
  script_name="$(basename "${script_path}")"
  if [[ "${script_name}" =~ ${milestone_pattern} ]]; then
    shell_milestone=$((shell_milestone + 1))
  elif [[ "${script_name}" =~ ${impl_pattern} ]]; then
    shell_impl=$((shell_impl + 1))
  else
    shell_other=$((shell_other + 1))
  fi
done

if [ "${output_format}" = "json" ]; then
  cat <<JSON
{
  "rust": {
    "test_files": ${rust_test_files},
    "tests": ${rust_test_count}
  },
  "shell": {
    "tests": ${shell_total},
    "implementation_or_runtime": ${shell_impl},
    "milestone_or_governance": ${shell_milestone},
    "other": ${shell_other}
  }
}
JSON
  exit 0
fi

cat <<REPORT
Test Inventory
==============
Rust tests
  files: ${rust_test_files}
  tests: ${rust_test_count}

Shell tests
  total: ${shell_total}
  implementation/runtime contracts: ${shell_impl}
  milestone/governance contracts:   ${shell_milestone}
  other:                            ${shell_other}

Tip:
  scripts/test-alpha-implementation-fast.sh
REPORT
