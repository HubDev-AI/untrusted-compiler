#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--matrix <path>] [--trend-note <path>] [--format <text|json>] [--fail-on-pending]

Checks strict closure evidence for milestone status gates (M9/M10/M11/M12/M13/M14/M15/M16/M17).
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_path=""
trend_note_path=""
fail_on_pending="false"
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
    --matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --trend-note)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      trend_note_path="$2"
      shift 2
      ;;
    --trend-note=*)
      trend_note_path="${1#--trend-note=}"
      shift
      ;;
    --fail-on-pending)
      fail_on_pending="true"
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

if [ -z "${matrix_path}" ]; then
  matrix_path="${repo_root}/benchmark-suite/results/summaries/compare-matrix.json"
fi
if [ -z "${trend_note_path}" ]; then
  trend_note_path="${repo_root}/docs/book/322-m13-first-trend-run-results-note.md"
fi

status_for() {
  if [ "$1" = "1" ]; then
    echo "PASS"
  else
    echo "PENDING"
  fi
}

render_evidence() {
  local value="$1"
  case "${value}" in
    "${repo_root}")
      echo "."
      ;;
    "${repo_root}"/*)
      echo "${value#${repo_root}/}"
      ;;
    *)
      echo "${value}"
      ;;
  esac
}

bool_has_release_gate=0
bool_has_release_gate_ci=0
bool_has_release_verifier_chain=0
bool_has_release_gate_closure_enforcement=0
bool_has_release_contract_smoke_workflow=0
bool_has_release_contract_smoke_ci_guard=0
bool_has_alpha_release_workflow_contract=0
bool_has_alpha_release_ci_guard=0
bool_has_cross_impl_matrix=0
bool_has_cross_impl_matrix_contract=0
bool_has_cross_impl_workflow_contract=0
bool_has_cross_impl_ci_guard=0
bool_has_zed_grammar_pin_ci_guard=0
bool_has_cli_command_ci_guard=0
bool_has_local_path_leak_ci_guard=0
bool_has_live_trend_entry=0
bool_has_trend_workflow_guards=0
bool_has_trend_workflow_artifact_upload=0
bool_has_benchmark_smoke_closure_contract=0
bool_has_trend_ci_guard=0
bool_has_explain_coverage_ci_guard=0
bool_has_replay_capture_ci_guard=0
bool_has_replay_compat_ci_guard=0
bool_has_replay_stub_registry_ci_guard=0
bool_has_replay_json_cli_ci_guard=0
bool_has_replay_execution_contract_guard=0
bool_has_m16_runtime_http_coverage_ci_guard=0
bool_has_m16_operator_smoke_script_ci_guard=0
bool_has_m16_runtime_smoke_workflow_contract=0
bool_has_m16_runtime_smoke_ci_guard=0
bool_has_m16_run_runtime_flag_ci_guard=0
bool_has_m17_operator_handoff_ci_guard=0
bool_has_m17_operator_bootstrap_ci_guard=0
bool_has_m17_operator_troubleshooting_ci_guard=0
gate_codes=()
gate_statuses=()
gate_labels=()
gate_evidence=()

[ -f "${repo_root}/scripts/release-alpha-gate.sh" ] && bool_has_release_gate=1
alpha_release_workflow_path="${repo_root}/.github/workflows/alpha-release-gate.yml"
[ -f "${alpha_release_workflow_path}" ] && bool_has_release_gate_ci=1
[ -f "${repo_root}/scripts/verify-release-promotion-inputs.sh" ] \
  && [ -f "${repo_root}/scripts/generate-release-publish-manifest.sh" ] \
  && [ -f "${repo_root}/scripts/verify-release-publish-manifest.sh" ] \
  && bool_has_release_verifier_chain=1

release_gate_script="${repo_root}/scripts/release-alpha-gate.sh"
if [ -f "${release_gate_script}" ] \
  && rg -q 'check-milestone-closure.sh' "${release_gate_script}" \
  && rg -q -- '--fail-on-pending' "${release_gate_script}"; then
  bool_has_release_gate_closure_enforcement=1
fi

if [ -f "${alpha_release_workflow_path}" ] \
  && rg -q 'scripts/release-alpha-gate.sh' "${alpha_release_workflow_path}" \
  && rg -q 'scripts/verify-release-promotion-inputs.sh' "${alpha_release_workflow_path}" \
  && rg -q 'scripts/generate-release-publish-manifest.sh' "${alpha_release_workflow_path}" \
  && rg -q 'scripts/verify-release-publish-manifest.sh' "${alpha_release_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/upload-artifact@v4' "${alpha_release_workflow_path}" \
  && rg -q 'name:[[:space:]]*alpha-release-gate-artifacts' "${alpha_release_workflow_path}" \
  && rg -q 'path:[[:space:]]*build/release-alpha-gate' "${alpha_release_workflow_path}"; then
  bool_has_alpha_release_workflow_contract=1
fi

release_contract_smoke_workflow_path="${repo_root}/.github/workflows/release-contract-smoke.yml"
if [ -f "${release_contract_smoke_workflow_path}" ] \
  && rg -q 'scripts/test-alpha-release-workflow-contract.sh' "${release_contract_smoke_workflow_path}" \
  && rg -q 'scripts/test-alpha-release-workflow-contract-guard.sh' "${release_contract_smoke_workflow_path}" \
  && rg -q 'scripts/test-verify-release-promotion-inputs.sh' "${release_contract_smoke_workflow_path}" \
  && rg -q 'scripts/test-generate-release-publish-manifest.sh' "${release_contract_smoke_workflow_path}" \
  && rg -q 'scripts/test-verify-release-publish-manifest.sh' "${release_contract_smoke_workflow_path}" \
  && rg -q 'scripts/test-release-contract-smoke-workflow-contract-guard.sh' "${release_contract_smoke_workflow_path}"; then
  bool_has_release_contract_smoke_workflow=1
fi

naming_lock_workflow_path="${repo_root}/.github/workflows/naming-lock.yml"
if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-release-contract-smoke-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-release-contract-smoke-workflow-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_release_contract_smoke_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-alpha-release-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-alpha-release-workflow-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_alpha_release_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-benchmark-cross-impl-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-benchmark-cross-impl-workflow-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_cross_impl_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-benchmark-trend-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-benchmark-trend-workflow-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_trend_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-check-sec4-explain-audit-coverage.sh' "${naming_lock_workflow_path}"; then
  bool_has_explain_coverage_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-replay-capture-contract.sh' "${naming_lock_workflow_path}"; then
  bool_has_replay_capture_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-replay-capture-compat.sh' "${naming_lock_workflow_path}"; then
  bool_has_replay_compat_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-replay-stub-registry-contract.sh' "${naming_lock_workflow_path}"; then
  bool_has_replay_stub_registry_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-replay-cli-json-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-replay-cli-json-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_replay_json_cli_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-m16-runtime-http-coverage.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-m16-runtime-http-coverage-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_m16_runtime_http_coverage_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-smoke-sec4-run-hello-api-script-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_m16_operator_smoke_script_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-runtime-smoke-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-runtime-smoke-workflow-contract-guard.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-check-runtime-smoke-artifacts.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-build-runtime-smoke-branch-index.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-check-runtime-smoke-bundle.sh' "${naming_lock_workflow_path}"; then
  bool_has_m16_runtime_smoke_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-sec4-run-runtime-flag-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-sec4-run-runtime-flag-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_m16_run_runtime_flag_ci_guard=1
fi

m17_handoff_test_script="${repo_root}/scripts/test-check-m17-operator-handoff-readiness.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_handoff_test_script}" ] \
  && rg -q 'scripts/test-check-m17-operator-handoff-readiness.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_handoff_ci_guard=1
fi

m17_bootstrap_test_script="${repo_root}/scripts/test-run-m17-operator-bootstrap.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_bootstrap_test_script}" ] \
  && rg -q 'scripts/test-run-m17-operator-bootstrap.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_bootstrap_ci_guard=1
fi

m17_troubleshooting_test_script="${repo_root}/scripts/test-print-m17-operator-troubleshooting-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_troubleshooting_test_script}" ] \
  && rg -q 'scripts/test-print-m17-operator-troubleshooting-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_troubleshooting_ci_guard=1
fi

replay_json_contract_script="${repo_root}/scripts/test-replay-cli-json-contract.sh"
replay_json_guard_script="${repo_root}/scripts/test-replay-cli-json-contract-guard.sh"
if [ -f "${replay_json_contract_script}" ] \
  && [ -f "${replay_json_guard_script}" ] \
  && rg -Fq '"mockExecutionCounts"' "${replay_json_contract_script}" \
  && rg -Fq '"mockExecutionTraces"' "${replay_json_contract_script}" \
  && rg -q 'mockExecutionCounts key is missing' "${replay_json_guard_script}" \
  && rg -q 'mockExecutionTraces key is missing' "${replay_json_guard_script}"; then
  bool_has_replay_execution_contract_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-zed-grammar-pin.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-zed-grammar-pin-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_zed_grammar_pin_ci_guard=1
fi

runtime_smoke_workflow_path="${repo_root}/.github/workflows/runtime-smoke.yml"
if [ -f "${runtime_smoke_workflow_path}" ] \
  && rg -q 'pull_request:' "${runtime_smoke_workflow_path}" \
  && rg -q 'push:' "${runtime_smoke_workflow_path}" \
  && rg -q 'branches:' "${runtime_smoke_workflow_path}" \
  && rg -q -- '- main' "${runtime_smoke_workflow_path}" \
  && rg -q 'runs-on:[[:space:]]*ubuntu-latest' "${runtime_smoke_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/checkout@v4' "${runtime_smoke_workflow_path}" \
  && rg -q 'scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default' "${runtime_smoke_workflow_path}" \
  && rg -q 'scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body' "${runtime_smoke_workflow_path}" \
  && rg -q 'scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json' "${runtime_smoke_workflow_path}" \
  && rg -q 'if:[[:space:]]*always\(\)' "${runtime_smoke_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/upload-artifact@v4' "${runtime_smoke_workflow_path}" \
  && rg -q 'name:[[:space:]]*runtime-smoke-artifacts' "${runtime_smoke_workflow_path}" \
  && rg -q 'path:[[:space:]]*build/runtime-smoke' "${runtime_smoke_workflow_path}"; then
  bool_has_m16_runtime_smoke_workflow_contract=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-sec4-cli-command-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-sec4-cli-command-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_cli_command_ci_guard=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-check-no-local-path-leaks.sh' "${naming_lock_workflow_path}"; then
  bool_has_local_path_leak_ci_guard=1
fi

if [ -f "${matrix_path}" ]; then
  if jq -e '
    .endpoints as $eps
    | ($eps | type == "array")
    and ($eps | length > 0)
    and all($eps[]; [ .compared[]?.impl ] as $impls
      | ($impls | index("sec4") != null)
      and ($impls | index("go") != null)
      and ($impls | index("node") != null)
      and ($impls | index("rust") != null))
  ' "${matrix_path}" >/dev/null 2>&1; then
    bool_has_cross_impl_matrix=1
  fi

  if jq -e '
    .endpoints as $eps
    | ($eps | type == "array")
    and ($eps | length > 0)
    and all($eps[];
      . as $entry
      | ($entry.endpoint | type == "string" and length > 0)
      and ($entry.compared | type == "array" and length > 0)
      and ($entry.leader | type == "object")
      and ($entry.leader.endpoint == $entry.endpoint)
      and ($entry.leader.impl | type == "string" and length > 0)
      and ($entry.leader.p99 | type == "string" and length > 0)
      and (if ($entry.leader | has("loadGenerator")) then ($entry.leader.loadGenerator | type == "string" and length > 0) else true end)
      and (if ($entry.leader | has("constantRate")) then ($entry.leader.constantRate | type == "boolean") else true end)
      and all($entry.compared[]; .endpoint == $entry.endpoint)
      and (
        $entry.leader as $leader
        | any($entry.compared[];
          .impl == $leader.impl
          and .endpoint == $leader.endpoint
          and .targetRps == $leader.targetRps
          and .requestsPerSec == $leader.requestsPerSec
          and .p99 == $leader.p99
          and ((if has("loadGenerator") then .loadGenerator else "wrk2" end) == (if ($leader | has("loadGenerator")) then $leader.loadGenerator else "wrk2" end))
          and ((if has("constantRate") then .constantRate else true end) == (if ($leader | has("constantRate")) then $leader.constantRate else true end))
        )
      )
    )
  ' "${matrix_path}" >/dev/null 2>&1; then
    bool_has_cross_impl_matrix_contract=1
  fi
fi

cross_impl_workflow_path="${repo_root}/.github/workflows/benchmark-cross-impl-evidence.yml"
if [ -f "${cross_impl_workflow_path}" ] \
  && rg -q 'benchmark-suite/scripts/run_comparison_matrix.sh' "${cross_impl_workflow_path}" \
  && rg -q -- '--impls sec4,node,go,rust' "${cross_impl_workflow_path}" \
  && rg -q -- '--endpoints ping,decode' "${cross_impl_workflow_path}" \
  && rg -q 'scripts/check-benchmark-evidence-quality.sh' "${cross_impl_workflow_path}" \
  && rg -q -- '--fail-on-warning' "${cross_impl_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/upload-artifact@v4' "${cross_impl_workflow_path}" \
  && rg -q 'name:[[:space:]]*benchmark-cross-impl-evidence' "${cross_impl_workflow_path}" \
  && rg -q 'path:[[:space:]]*benchmark-suite/results' "${cross_impl_workflow_path}"; then
  bool_has_cross_impl_workflow_contract=1
fi

if [ -f "${trend_note_path}" ]; then
  if rg -q '^## Trend Entry \([0-9]{4}-[0-9]{2}-[0-9]{2}\)$' "${trend_note_path}"; then
    bool_has_live_trend_entry=1
  fi
fi

trend_workflow_path="${repo_root}/.github/workflows/benchmark-trend.yml"
if [ -f "${trend_workflow_path}" ] \
  && rg -q '^[[:space:]]*schedule:' "${trend_workflow_path}" \
  && rg -q 'cron:' "${trend_workflow_path}" \
  && rg -q 'scripts/check-benchmark-evidence-quality.sh' "${trend_workflow_path}" \
  && rg -q -- '--fail-on-warning' "${trend_workflow_path}" \
  && rg -q 'benchmark-suite/scripts/check_regression_thresholds.sh' "${trend_workflow_path}"; then
  bool_has_trend_workflow_guards=1
fi

if [ -f "${trend_workflow_path}" ] \
  && rg -q 'uses:[[:space:]]*actions/upload-artifact@v4' "${trend_workflow_path}" \
  && rg -q 'name:[[:space:]]*benchmark-trend-' "${trend_workflow_path}" \
  && rg -q 'path:[[:space:]]*benchmark-suite/results' "${trend_workflow_path}"; then
  bool_has_trend_workflow_artifact_upload=1
fi

benchmark_smoke_workflow_path="${repo_root}/.github/workflows/benchmark-smoke.yml"
if [ -f "${benchmark_smoke_workflow_path}" ] \
  && rg -q 'scripts/test-benchmark-smoke-closure-gate.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-benchmark-smoke-closure-gate-guard.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-benchmark-cross-impl-workflow-contract.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-benchmark-cross-impl-workflow-contract-guard.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-benchmark-trend-workflow-contract.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-benchmark-trend-workflow-contract-guard.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/test-check-milestone-closure.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q 'scripts/check-milestone-closure.sh' "${benchmark_smoke_workflow_path}" \
  && rg -q -- '--fail-on-pending' "${benchmark_smoke_workflow_path}"; then
  bool_has_benchmark_smoke_closure_contract=1
fi

pending_count=0

emit_check() {
  local code="$1"
  local label="$2"
  local pass="$3"
  local evidence="$4"
  local rendered_evidence
  local status
  rendered_evidence="$(render_evidence "${evidence}")"
  status="$(status_for "${pass}")"
  if [ "${pass}" -eq 0 ]; then
    pending_count=$((pending_count + 1))
  fi
  gate_codes+=("${code}")
  gate_statuses+=("${status}")
  gate_labels+=("${label}")
  gate_evidence+=("${rendered_evidence}")
  if [ "${output_format}" = "text" ]; then
    printf '%-6s %-8s %-64s %s\n' "${code}" "${status}" "${label}" "${rendered_evidence}"
  fi
}

if [ "${output_format}" = "text" ]; then
  echo "Milestone Closure Audit"
  echo "repo: $(render_evidence "${repo_root}")"
  echo
  printf '%-6s %-8s %-64s %s\n' "Gate" "Status" "Check" "Evidence"
  printf '%-6s %-8s %-64s %s\n' "-----" "--------" "----------------------------------------------------------------" "--------"
fi

emit_check "M9-A" "release gate script exists" "${bool_has_release_gate}" "scripts/release-alpha-gate.sh"
emit_check "M9-B" "release gate workflow exists" "${bool_has_release_gate_ci}" ".github/workflows/alpha-release-gate.yml"
emit_check "M9-C" "promotion verifier/manifest chain exists" "${bool_has_release_verifier_chain}" "scripts/verify-release-promotion-inputs.sh + publish-manifest scripts"
emit_check "M9-D" "release gate enforces strict milestone closure" "${bool_has_release_gate_closure_enforcement}" "scripts/release-alpha-gate.sh"
emit_check "M9-E" "release-contract-smoke workflow keeps release verifier/publish + guard tests" "${bool_has_release_contract_smoke_workflow}" "${release_contract_smoke_workflow_path}"
emit_check "M9-F" "naming-lock CI enforces release-contract-smoke contract + guard tests" "${bool_has_release_contract_smoke_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M9-G" "alpha-release workflow keeps release/promotion/publish/upload contract" "${bool_has_alpha_release_workflow_contract}" "${alpha_release_workflow_path}"
emit_check "M9-H" "naming-lock CI enforces alpha-release workflow contract + guard tests" "${bool_has_alpha_release_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M10-A" "cross-impl matrix includes sec4/go/node/rust for each endpoint" "${bool_has_cross_impl_matrix}" "${matrix_path}"
emit_check "M10-B" "cross-impl matrix row contract is aligned" "${bool_has_cross_impl_matrix_contract}" "${matrix_path}"
emit_check "M10-C" "cross-impl workflow enforces scoped run + strict quality + artifact upload" "${bool_has_cross_impl_workflow_contract}" "${cross_impl_workflow_path}"
emit_check "M10-D" "naming-lock CI enforces cross-impl workflow contract + guard tests" "${bool_has_cross_impl_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M11-A" "naming-lock CI enforces zed grammar pin contract + guard tests" "${bool_has_zed_grammar_pin_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M12-A" "naming-lock CI enforces sec4 CLI command contract + guard tests" "${bool_has_cli_command_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M12-B" "naming-lock CI enforces local path leak guard test" "${bool_has_local_path_leak_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M13-A" "trend note contains at least one live Trend Entry block" "${bool_has_live_trend_entry}" "${trend_note_path}"
emit_check "M13-B" "benchmark trend workflow has strict quality + regression guards" "${bool_has_trend_workflow_guards}" "${trend_workflow_path}"
emit_check "M13-C" "benchmark trend workflow uploads trend artifacts" "${bool_has_trend_workflow_artifact_upload}" "${trend_workflow_path}"
emit_check "M13-D" "benchmark-smoke workflow enforces closure + cross-impl/trend contract guards + strict closure audit" "${bool_has_benchmark_smoke_closure_contract}" "${benchmark_smoke_workflow_path}"
emit_check "M13-E" "naming-lock CI enforces benchmark-trend workflow contract + guard tests" "${bool_has_trend_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M13-F" "naming-lock CI enforces sec4 explain audit-coverage contract test" "${bool_has_explain_coverage_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M14-A" "naming-lock CI enforces replay capture contract test" "${bool_has_replay_capture_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M14-B" "naming-lock CI enforces replay capture compatibility test" "${bool_has_replay_compat_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M14-C" "naming-lock CI enforces replay stub registry contract test" "${bool_has_replay_stub_registry_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M14-D" "naming-lock CI enforces replay CLI json contract + guard tests" "${bool_has_replay_json_cli_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M15-A" "replay CLI json contract guards execution fields + missing-key fixtures" "${bool_has_replay_execution_contract_guard}" "scripts/test-replay-cli-json-contract.sh + scripts/test-replay-cli-json-contract-guard.sh"
emit_check "M16-A" "naming-lock CI enforces M16 runtime HTTP coverage contract + guard tests" "${bool_has_m16_runtime_http_coverage_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M16-B" "naming-lock CI enforces sec4 run hello-api smoke script contract + guard tests" "${bool_has_m16_operator_smoke_script_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M16-C" "runtime-smoke workflow executes dual-branch sec4 run smoke + artifact validation/index/upload on pull_request + main push" "${bool_has_m16_runtime_smoke_workflow_contract}" "${runtime_smoke_workflow_path}"
emit_check "M16-D" "naming-lock CI enforces runtime-smoke workflow contract + guard tests + artifact checker/index/bundle tests" "${bool_has_m16_runtime_smoke_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M16-E" "naming-lock CI enforces sec4 run runtime-flag contract + guard tests" "${bool_has_m16_run_runtime_flag_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-A" "naming-lock CI enforces M17 operator handoff readiness checker" "${bool_has_m17_operator_handoff_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-B" "naming-lock CI enforces M17 operator bootstrap profile helper" "${bool_has_m17_operator_bootstrap_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-C" "naming-lock CI enforces M17 operator troubleshooting matrix" "${bool_has_m17_operator_troubleshooting_ci_guard}" "${naming_lock_workflow_path}"

if [ "${output_format}" = "json" ]; then
  gates_json='[]'
  for i in "${!gate_codes[@]}"; do
    gates_json="$(
      jq \
        --arg gate "${gate_codes[i]}" \
        --arg status "${gate_statuses[i]}" \
        --arg check "${gate_labels[i]}" \
        --arg evidence "${gate_evidence[i]}" \
        '. + [{gate: $gate, status: $status, check: $check, evidence: $evidence}]' \
        <<<"${gates_json}"
    )"
  done

  if [ "${pending_count}" -eq 0 ]; then
    overall_status="PASS"
    overall_message="all tracked closure checks satisfied"
  else
    overall_status="PENDING"
    overall_message="${pending_count} check(s) not yet satisfied"
  fi

  jq \
    -n \
    --arg repo "$(render_evidence "${repo_root}")" \
    --arg overall "${overall_status}" \
    --arg message "${overall_message}" \
    --argjson pendingCount "${pending_count}" \
    --argjson gates "${gates_json}" \
    '{repo: $repo, overall: $overall, message: $message, pendingCount: $pendingCount, gates: $gates}'
else
  echo
  if [ "${pending_count}" -eq 0 ]; then
    echo "overall: PASS (all tracked closure checks satisfied)"
  else
    echo "overall: PENDING (${pending_count} check(s) not yet satisfied)"
  fi
fi

if [ "${fail_on_pending}" = "true" ]; then
  if [ "${pending_count}" -eq 0 ]; then
    exit 0
  fi
  exit 1
fi

exit 0
