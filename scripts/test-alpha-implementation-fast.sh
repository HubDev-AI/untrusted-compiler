#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

dry_run="false"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    -h|--help)
      cat <<'USAGE'
usage: scripts/test-alpha-implementation-fast.sh [--dry-run]

Runs a focused, implementation-heavy validation pass without milestone closure bundles.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

step=1
run_step() {
  local label="$1"
  shift
  printf '\n[%d] %s\n' "${step}" "${label}"
  if [ "${dry_run}" = "true" ]; then
    printf '    '
    printf '%q ' "$@"
    printf '\n'
  else
    "$@"
  fi
  step=$((step + 1))
}

run_step "Core C backend ABI/runtime contract coverage" \
  cargo test -p sec4-core --test c_backend

run_step "CLI command behavior coverage" \
  cargo test -p sec4 --test commands

run_step "Alpha smoke path (init/check/build/run)" \
  cargo test -p sec4 --test alpha_smoke

if command -v clang >/dev/null 2>&1; then
  run_step "Targeted c-bin req/res intrinsic compile contract (clang-gated)" \
    cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available
else
  printf '\n[%d] Skip clang-gated json_output targeted check (clang not found)\n' "${step}"
  step=$((step + 1))
fi

run_step "CLI contract lock" \
  scripts/test-sec4-cli-command-contract.sh

run_step "Run flag contract lock" \
  scripts/test-sec4-run-runtime-flag-contract.sh

run_step "Hello API smoke script contract lock" \
  scripts/test-smoke-sec4-run-hello-api-script-contract.sh

run_step "Audit explain coverage lock" \
  scripts/test-check-sec4-explain-audit-coverage.sh

run_step "Path leak hygiene lock" \
  scripts/test-check-no-local-path-leaks.sh

printf '\nFast implementation suite completed.\n'
