#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--matrix <path>] [--trend-note <path>] [--format <text|json>] [--fail-on-pending]

Checks strict closure evidence for milestone status gates (M9/M10/M11/M12/M13/M14/M15/M16/M17/M18/M19/M20/M21/M22/M23/M24/M25/M26/M27/M28/M29).
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
bool_has_m17_operator_quickstart_ci_guard=0
bool_has_m17_operator_ci_smoke_ci_guard=0
bool_has_m17_operator_handoff_workflow_contract=0
bool_has_m17_operator_handoff_workflow_ci_guard=0
bool_has_m17_operator_artifact_inspector_ci_guard=0
bool_has_m17_operator_summary_ci_guard=0
bool_has_m17_operator_release_packet_ci_guard=0
bool_has_m17_operator_playbook_ci_guard=0
bool_has_m17_operator_clean_clone_ci_guard=0
bool_has_m18_kickoff_brief_ci_guard=0
bool_has_m18_priority_matrix_ci_guard=0
bool_has_m18_slice_selector_ci_guard=0
bool_has_m18_editor_contract_ci_guard=0
bool_has_m18_release_publish_integrity_ci_guard=0
bool_has_m18_runtime_track_runner_ci_guard=0
bool_has_m18_track_convergence_summary_ci_guard=0
bool_has_m18_transition_handoff_packet_ci_guard=0
bool_has_m18_closure_report_ci_guard=0
bool_has_m19_kickoff_brief_ci_guard=0
bool_has_m19_priority_matrix_ci_guard=0
bool_has_m19_slice_selector_ci_guard=0
bool_has_m19_runtime_hardening_runner_ci_guard=0
bool_has_m19_executed_slice_convergence_summary_ci_guard=0
bool_has_m19_transition_handoff_packet_ci_guard=0
bool_has_m19_closure_report_ci_guard=0
bool_has_m20_kickoff_brief_ci_guard=0
bool_has_m20_priority_matrix_ci_guard=0
bool_has_m20_slice_selector_ci_guard=0
bool_has_m20_runtime_hardening_runner_ci_guard=0
bool_has_m20_executed_slice_convergence_summary_ci_guard=0
bool_has_m20_transition_handoff_packet_ci_guard=0
bool_has_m20_closure_report_ci_guard=0
bool_has_m21_kickoff_brief_ci_guard=0
bool_has_m21_priority_matrix_ci_guard=0
bool_has_m21_slice_selector_ci_guard=0
bool_has_m21_runtime_hardening_runner_ci_guard=0
bool_has_m21_executed_slice_convergence_summary_ci_guard=0
bool_has_m21_transition_handoff_packet_ci_guard=0
bool_has_m21_closure_report_ci_guard=0
bool_has_m22_kickoff_brief_ci_guard=0
bool_has_m22_priority_matrix_ci_guard=0
bool_has_m22_slice_selector_ci_guard=0
bool_has_m22_runtime_hardening_runner_ci_guard=0
bool_has_m22_executed_slice_convergence_summary_ci_guard=0
bool_has_m22_transition_handoff_packet_ci_guard=0
bool_has_m22_closure_report_ci_guard=0
bool_has_m23_kickoff_brief_ci_guard=0
bool_has_m23_priority_matrix_ci_guard=0
bool_has_m23_slice_selector_ci_guard=0
bool_has_m23_runtime_hardening_runner_ci_guard=0
bool_has_m23_executed_slice_convergence_summary_ci_guard=0
bool_has_m23_transition_handoff_packet_ci_guard=0
bool_has_m23_closure_report_ci_guard=0
bool_has_m24_kickoff_brief_ci_guard=0
bool_has_m24_priority_matrix_ci_guard=0
bool_has_m24_slice_selector_ci_guard=0
bool_has_m24_runtime_hardening_runner_ci_guard=0
bool_has_m24_executed_slice_convergence_summary_ci_guard=0
bool_has_m24_transition_handoff_packet_ci_guard=0
bool_has_m24_closure_report_ci_guard=0
bool_has_m25_kickoff_brief_ci_guard=0
bool_has_m25_priority_matrix_ci_guard=0
bool_has_m25_slice_selector_ci_guard=0
bool_has_m25_runtime_hardening_runner_ci_guard=0
bool_has_m25_executed_slice_convergence_summary_ci_guard=0
bool_has_m25_transition_handoff_packet_ci_guard=0
bool_has_m25_closure_report_ci_guard=0
bool_has_m26_kickoff_brief_ci_guard=0
bool_has_m26_priority_matrix_ci_guard=0
bool_has_m26_slice_selector_ci_guard=0
bool_has_m26_runtime_hardening_runner_ci_guard=0
bool_has_m26_executed_slice_convergence_summary_ci_guard=0
bool_has_m26_transition_handoff_packet_ci_guard=0
bool_has_m26_closure_report_ci_guard=0
bool_has_m27_kickoff_brief_ci_guard=0
bool_has_m27_priority_matrix_ci_guard=0
bool_has_m27_slice_selector_ci_guard=0
bool_has_m27_runtime_hardening_runner_ci_guard=0
bool_has_m27_executed_slice_convergence_summary_ci_guard=0
bool_has_m27_transition_handoff_packet_ci_guard=0
bool_has_m27_closure_report_ci_guard=0
bool_has_m28_kickoff_brief_ci_guard=0
bool_has_m28_priority_matrix_ci_guard=0
bool_has_m28_slice_selector_ci_guard=0
bool_has_m28_runtime_hardening_runner_ci_guard=0
bool_has_m28_executed_slice_convergence_summary_ci_guard=0
bool_has_m28_transition_handoff_packet_ci_guard=0
bool_has_m28_closure_report_ci_guard=0
bool_has_m29_kickoff_brief_ci_guard=0
bool_has_m29_priority_matrix_ci_guard=0
bool_has_m29_runtime_destub_plan_ci_guard=0
bool_has_m29_runtime_destub_runner_ci_guard=0
bool_has_m29_executed_slice_convergence_summary_ci_guard=0
bool_has_m29_transition_handoff_packet_ci_guard=0
bool_has_m29_closure_report_ci_guard=0
bool_has_m30_kickoff_brief_ci_guard=0
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

m17_quickstart_test_script="${repo_root}/scripts/test-run-m17-operator-handoff-quickstart.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_quickstart_test_script}" ] \
  && rg -q 'scripts/test-run-m17-operator-handoff-quickstart.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_quickstart_ci_guard=1
fi

m17_ci_smoke_test_script="${repo_root}/scripts/test-run-m17-operator-handoff-ci-smoke.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_ci_smoke_test_script}" ] \
  && rg -q 'scripts/test-run-m17-operator-handoff-ci-smoke.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_ci_smoke_ci_guard=1
fi

operator_handoff_workflow_path="${repo_root}/.github/workflows/operator-handoff-smoke.yml"
if [ -f "${operator_handoff_workflow_path}" ] \
  && rg -q 'pull_request:' "${operator_handoff_workflow_path}" \
  && rg -q 'push:' "${operator_handoff_workflow_path}" \
  && rg -q 'branches:' "${operator_handoff_workflow_path}" \
  && rg -q -- '- main' "${operator_handoff_workflow_path}" \
  && rg -q 'runs-on:[[:space:]]*ubuntu-latest' "${operator_handoff_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/checkout@v4' "${operator_handoff_workflow_path}" \
  && rg -q 'scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke' "${operator_handoff_workflow_path}" \
  && rg -q 'if:[[:space:]]*always\(\)' "${operator_handoff_workflow_path}" \
  && rg -q 'uses:[[:space:]]*actions/upload-artifact@v4' "${operator_handoff_workflow_path}" \
  && rg -q 'name:[[:space:]]*operator-handoff-smoke-artifacts' "${operator_handoff_workflow_path}" \
  && rg -q 'path:[[:space:]]*build/operator-handoff-smoke' "${operator_handoff_workflow_path}"; then
  bool_has_m17_operator_handoff_workflow_contract=1
fi

if [ -f "${naming_lock_workflow_path}" ] \
  && rg -q 'scripts/test-operator-handoff-workflow-contract.sh' "${naming_lock_workflow_path}" \
  && rg -q 'scripts/test-operator-handoff-workflow-contract-guard.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_handoff_workflow_ci_guard=1
fi

m17_artifact_inspector_test_script="${repo_root}/scripts/test-inspect-m17-operator-handoff-artifacts.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_artifact_inspector_test_script}" ] \
  && rg -q 'scripts/test-inspect-m17-operator-handoff-artifacts.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_artifact_inspector_ci_guard=1
fi

m17_summary_test_script="${repo_root}/scripts/test-summarize-m17-operator-handoff-readiness.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_summary_test_script}" ] \
  && rg -q 'scripts/test-summarize-m17-operator-handoff-readiness.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_summary_ci_guard=1
fi

m17_release_packet_test_script="${repo_root}/scripts/test-build-m17-operator-release-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_release_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m17-operator-release-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_release_packet_ci_guard=1
fi

m17_playbook_test_script="${repo_root}/scripts/test-check-m17-operator-handoff-playbook.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_playbook_test_script}" ] \
  && rg -q 'scripts/test-check-m17-operator-handoff-playbook.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_playbook_ci_guard=1
fi

m17_clean_clone_test_script="${repo_root}/scripts/test-run-m17-operator-clean-clone-rehearsal.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m17_clean_clone_test_script}" ] \
  && rg -q 'scripts/test-run-m17-operator-clean-clone-rehearsal.sh' "${naming_lock_workflow_path}"; then
  bool_has_m17_operator_clean_clone_ci_guard=1
fi

m18_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m18-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m18-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_kickoff_brief_ci_guard=1
fi

m18_priority_matrix_test_script="${repo_root}/scripts/test-build-m18-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m18-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_priority_matrix_ci_guard=1
fi

m18_slice_selector_test_script="${repo_root}/scripts/test-select-m18-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m18-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_slice_selector_ci_guard=1
fi

m18_editor_contract_test_script="${repo_root}/scripts/test-m18-editor-contract-expansion.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_editor_contract_test_script}" ] \
  && rg -q 'scripts/test-m18-editor-contract-expansion.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_editor_contract_ci_guard=1
fi

m18_release_publish_integrity_test_script="${repo_root}/scripts/test-m18-release-publish-integrity.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_release_publish_integrity_test_script}" ] \
  && rg -q 'scripts/test-m18-release-publish-integrity.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_release_publish_integrity_ci_guard=1
fi

m18_runtime_track_runner_test_script="${repo_root}/scripts/test-run-m18-runtime-track.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_runtime_track_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m18-runtime-track.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_runtime_track_runner_ci_guard=1
fi

m18_track_convergence_summary_test_script="${repo_root}/scripts/test-build-m18-track-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_track_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m18-track-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_track_convergence_summary_ci_guard=1
fi

m18_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m18-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m18-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_transition_handoff_packet_ci_guard=1
fi

m18_closure_report_test_script="${repo_root}/scripts/test-build-m18-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m18_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m18-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m18_closure_report_ci_guard=1
fi

m19_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m19-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m19-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_kickoff_brief_ci_guard=1
fi

m19_priority_matrix_test_script="${repo_root}/scripts/test-build-m19-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m19-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_priority_matrix_ci_guard=1
fi

m19_slice_selector_test_script="${repo_root}/scripts/test-select-m19-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m19-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_slice_selector_ci_guard=1
fi

m19_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m19-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m19-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_runtime_hardening_runner_ci_guard=1
fi

m19_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m19-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m19-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_executed_slice_convergence_summary_ci_guard=1
fi

m19_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m19-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m19-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_transition_handoff_packet_ci_guard=1
fi

m19_closure_report_test_script="${repo_root}/scripts/test-build-m19-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m19_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m19-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m19_closure_report_ci_guard=1
fi

m20_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m20-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m20-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_kickoff_brief_ci_guard=1
fi

m20_priority_matrix_test_script="${repo_root}/scripts/test-build-m20-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m20-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_priority_matrix_ci_guard=1
fi

m20_slice_selector_test_script="${repo_root}/scripts/test-select-m20-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m20-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_slice_selector_ci_guard=1
fi

m20_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m20-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m20-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_runtime_hardening_runner_ci_guard=1
fi

m20_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m20-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m20-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_executed_slice_convergence_summary_ci_guard=1
fi

m20_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m20-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m20-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_transition_handoff_packet_ci_guard=1
fi

m20_closure_report_test_script="${repo_root}/scripts/test-build-m20-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m20_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m20-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m20_closure_report_ci_guard=1
fi

m21_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m21-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m21-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_kickoff_brief_ci_guard=1
fi

m21_priority_matrix_test_script="${repo_root}/scripts/test-build-m21-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m21-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_priority_matrix_ci_guard=1
fi

m21_slice_selector_test_script="${repo_root}/scripts/test-select-m21-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m21-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_slice_selector_ci_guard=1
fi

m21_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m21-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m21-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_runtime_hardening_runner_ci_guard=1
fi

m21_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m21-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m21-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_executed_slice_convergence_summary_ci_guard=1
fi

m21_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m21-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m21-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_transition_handoff_packet_ci_guard=1
fi

m21_closure_report_test_script="${repo_root}/scripts/test-build-m21-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m21_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m21-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m21_closure_report_ci_guard=1
fi

m22_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m22-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m22-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_kickoff_brief_ci_guard=1
fi

m22_priority_matrix_test_script="${repo_root}/scripts/test-build-m22-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m22-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_priority_matrix_ci_guard=1
fi

m22_slice_selector_test_script="${repo_root}/scripts/test-select-m22-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m22-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_slice_selector_ci_guard=1
fi

m22_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m22-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m22-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_runtime_hardening_runner_ci_guard=1
fi

m22_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m22-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m22-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_executed_slice_convergence_summary_ci_guard=1
fi

m22_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m22-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m22-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_transition_handoff_packet_ci_guard=1
fi

m22_closure_report_test_script="${repo_root}/scripts/test-build-m22-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m22_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m22-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m22_closure_report_ci_guard=1
fi

m23_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m23-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m23-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_kickoff_brief_ci_guard=1
fi

m23_priority_matrix_test_script="${repo_root}/scripts/test-build-m23-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m23-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_priority_matrix_ci_guard=1
fi

m23_slice_selector_test_script="${repo_root}/scripts/test-select-m23-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m23-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_slice_selector_ci_guard=1
fi

m23_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m23-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m23-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_runtime_hardening_runner_ci_guard=1
fi

m23_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m23-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m23-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_executed_slice_convergence_summary_ci_guard=1
fi

m23_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m23-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m23-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_transition_handoff_packet_ci_guard=1
fi

m23_closure_report_test_script="${repo_root}/scripts/test-build-m23-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m23_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m23-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m23_closure_report_ci_guard=1
fi

m24_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m24-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m24-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_kickoff_brief_ci_guard=1
fi

m24_priority_matrix_test_script="${repo_root}/scripts/test-build-m24-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m24-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_priority_matrix_ci_guard=1
fi

m24_slice_selector_test_script="${repo_root}/scripts/test-select-m24-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m24-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_slice_selector_ci_guard=1
fi

m24_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m24-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m24-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_runtime_hardening_runner_ci_guard=1
fi

m24_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m24-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m24-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_executed_slice_convergence_summary_ci_guard=1
fi

m24_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m24-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m24-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_transition_handoff_packet_ci_guard=1
fi

m24_closure_report_test_script="${repo_root}/scripts/test-build-m24-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m24_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m24-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m24_closure_report_ci_guard=1
fi

m25_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m25-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m25-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_kickoff_brief_ci_guard=1
fi

m25_priority_matrix_test_script="${repo_root}/scripts/test-build-m25-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m25-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_priority_matrix_ci_guard=1
fi

m25_slice_selector_test_script="${repo_root}/scripts/test-select-m25-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m25-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_slice_selector_ci_guard=1
fi

m25_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m25-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m25-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_runtime_hardening_runner_ci_guard=1
fi

m25_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m25-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m25-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_executed_slice_convergence_summary_ci_guard=1
fi

m25_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m25-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m25-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_transition_handoff_packet_ci_guard=1
fi

m25_closure_report_test_script="${repo_root}/scripts/test-build-m25-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m25_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m25-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m25_closure_report_ci_guard=1
fi

m26_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m26-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m26-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_kickoff_brief_ci_guard=1
fi

m26_priority_matrix_test_script="${repo_root}/scripts/test-build-m26-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m26-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_priority_matrix_ci_guard=1
fi

m26_slice_selector_test_script="${repo_root}/scripts/test-select-m26-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m26-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_slice_selector_ci_guard=1
fi

m26_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m26-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m26-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_runtime_hardening_runner_ci_guard=1
fi

m26_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m26-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m26-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_executed_slice_convergence_summary_ci_guard=1
fi

m26_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m26-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m26-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_transition_handoff_packet_ci_guard=1
fi

m26_closure_report_test_script="${repo_root}/scripts/test-build-m26-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m26_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m26-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m26_closure_report_ci_guard=1
fi

m27_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m27-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m27-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_kickoff_brief_ci_guard=1
fi

m27_priority_matrix_test_script="${repo_root}/scripts/test-build-m27-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m27-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_priority_matrix_ci_guard=1
fi

m27_slice_selector_test_script="${repo_root}/scripts/test-select-m27-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m27-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_slice_selector_ci_guard=1
fi

m27_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m27-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m27-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_runtime_hardening_runner_ci_guard=1
fi

m27_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m27-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m27-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_executed_slice_convergence_summary_ci_guard=1
fi

m27_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m27-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m27-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_transition_handoff_packet_ci_guard=1
fi

m27_closure_report_test_script="${repo_root}/scripts/test-build-m27-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m27_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m27-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m27_closure_report_ci_guard=1
fi

m28_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m28-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m28-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_kickoff_brief_ci_guard=1
fi

m28_priority_matrix_test_script="${repo_root}/scripts/test-build-m28-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m28-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_priority_matrix_ci_guard=1
fi

m28_slice_selector_test_script="${repo_root}/scripts/test-select-m28-next-slice.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_slice_selector_test_script}" ] \
  && rg -q 'scripts/test-select-m28-next-slice.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_slice_selector_ci_guard=1
fi

m28_runtime_hardening_runner_test_script="${repo_root}/scripts/test-run-m28-runtime-hardening.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_runtime_hardening_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m28-runtime-hardening.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_runtime_hardening_runner_ci_guard=1
fi

m28_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m28-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m28-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_executed_slice_convergence_summary_ci_guard=1
fi

m28_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m28-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m28-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_transition_handoff_packet_ci_guard=1
fi

m28_closure_report_test_script="${repo_root}/scripts/test-build-m28-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m28_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m28-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m28_closure_report_ci_guard=1
fi

m29_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m29-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m29-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_kickoff_brief_ci_guard=1
fi

m29_priority_matrix_test_script="${repo_root}/scripts/test-build-m29-priority-matrix.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_priority_matrix_test_script}" ] \
  && rg -q 'scripts/test-build-m29-priority-matrix.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_priority_matrix_ci_guard=1
fi

m29_runtime_destub_plan_test_script="${repo_root}/scripts/test-plan-m29-runtime-destub.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_runtime_destub_plan_test_script}" ] \
  && rg -q 'scripts/test-plan-m29-runtime-destub.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_runtime_destub_plan_ci_guard=1
fi

m29_runtime_destub_runner_test_script="${repo_root}/scripts/test-run-m29-runtime-destub.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_runtime_destub_runner_test_script}" ] \
  && rg -q 'scripts/test-run-m29-runtime-destub.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_runtime_destub_runner_ci_guard=1
fi

m29_executed_slice_convergence_summary_test_script="${repo_root}/scripts/test-build-m29-executed-slice-convergence-summary.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_executed_slice_convergence_summary_test_script}" ] \
  && rg -q 'scripts/test-build-m29-executed-slice-convergence-summary.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_executed_slice_convergence_summary_ci_guard=1
fi

m29_transition_handoff_packet_test_script="${repo_root}/scripts/test-build-m29-transition-handoff-packet.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_transition_handoff_packet_test_script}" ] \
  && rg -q 'scripts/test-build-m29-transition-handoff-packet.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_transition_handoff_packet_ci_guard=1
fi

m29_closure_report_test_script="${repo_root}/scripts/test-build-m29-closure-report.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m29_closure_report_test_script}" ] \
  && rg -q 'scripts/test-build-m29-closure-report.sh' "${naming_lock_workflow_path}"; then
  bool_has_m29_closure_report_ci_guard=1
fi

m30_kickoff_brief_test_script="${repo_root}/scripts/test-generate-m30-kickoff-brief.sh"
if [ -f "${naming_lock_workflow_path}" ] \
  && [ -f "${m30_kickoff_brief_test_script}" ] \
  && rg -q 'scripts/test-generate-m30-kickoff-brief.sh' "${naming_lock_workflow_path}"; then
  bool_has_m30_kickoff_brief_ci_guard=1
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
emit_check "M17-D" "naming-lock CI enforces M17 operator handoff quickstart" "${bool_has_m17_operator_quickstart_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-E" "naming-lock CI enforces M17 operator handoff CI smoke wrapper" "${bool_has_m17_operator_ci_smoke_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-F" "operator-handoff workflow executes CI smoke wrapper + artifact upload on pull_request + main push" "${bool_has_m17_operator_handoff_workflow_contract}" "${operator_handoff_workflow_path}"
emit_check "M17-G" "naming-lock CI enforces operator-handoff workflow contract + guard tests" "${bool_has_m17_operator_handoff_workflow_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-H" "naming-lock CI enforces M17 operator handoff artifact inspector" "${bool_has_m17_operator_artifact_inspector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-I" "naming-lock CI enforces M17 operator handoff readiness summary" "${bool_has_m17_operator_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-J" "naming-lock CI enforces M17 operator release packet builder" "${bool_has_m17_operator_release_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-K" "naming-lock CI enforces M17 final handoff playbook checker" "${bool_has_m17_operator_playbook_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M17-L" "naming-lock CI enforces M17 clean-clone rehearsal runner" "${bool_has_m17_operator_clean_clone_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-A" "naming-lock CI enforces M18 kickoff brief generator" "${bool_has_m18_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-B" "naming-lock CI enforces M18 priority matrix artifact" "${bool_has_m18_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-C" "naming-lock CI enforces M18 next-slice selector" "${bool_has_m18_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-D" "naming-lock CI enforces M18 editor contract expansion" "${bool_has_m18_editor_contract_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-E" "naming-lock CI enforces M18 release publish integrity contract expansion" "${bool_has_m18_release_publish_integrity_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-F" "naming-lock CI enforces M18 runtime-track execution runner" "${bool_has_m18_runtime_track_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-G" "naming-lock CI enforces M18 track convergence summary" "${bool_has_m18_track_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-H" "naming-lock CI enforces M18 transition handoff packet" "${bool_has_m18_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M18-I" "naming-lock CI enforces M18 closure report" "${bool_has_m18_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-A" "naming-lock CI enforces M19 kickoff brief" "${bool_has_m19_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-B" "naming-lock CI enforces M19 priority matrix" "${bool_has_m19_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-C" "naming-lock CI enforces M19 next-slice selector" "${bool_has_m19_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-D" "naming-lock CI enforces M19 runtime hardening runner" "${bool_has_m19_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-E" "naming-lock CI enforces M19 executed-slice convergence summary" "${bool_has_m19_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-F" "naming-lock CI enforces M19 transition handoff packet" "${bool_has_m19_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M19-G" "naming-lock CI enforces M19 closure report" "${bool_has_m19_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-A" "naming-lock CI enforces M20 kickoff brief" "${bool_has_m20_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-B" "naming-lock CI enforces M20 priority matrix" "${bool_has_m20_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-C" "naming-lock CI enforces M20 next-slice selector" "${bool_has_m20_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-D" "naming-lock CI enforces M20 runtime hardening runner" "${bool_has_m20_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-E" "naming-lock CI enforces M20 executed-slice convergence summary" "${bool_has_m20_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-F" "naming-lock CI enforces M20 transition handoff packet" "${bool_has_m20_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M20-G" "naming-lock CI enforces M20 closure report" "${bool_has_m20_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-A" "naming-lock CI enforces M21 kickoff brief" "${bool_has_m21_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-B" "naming-lock CI enforces M21 priority matrix" "${bool_has_m21_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-C" "naming-lock CI enforces M21 next-slice selector" "${bool_has_m21_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-D" "naming-lock CI enforces M21 runtime hardening runner" "${bool_has_m21_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-E" "naming-lock CI enforces M21 executed-slice convergence summary" "${bool_has_m21_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-F" "naming-lock CI enforces M21 transition handoff packet" "${bool_has_m21_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M21-G" "naming-lock CI enforces M21 closure report" "${bool_has_m21_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-A" "naming-lock CI enforces M22 kickoff brief" "${bool_has_m22_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-B" "naming-lock CI enforces M22 priority matrix" "${bool_has_m22_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-C" "naming-lock CI enforces M22 next-slice selector" "${bool_has_m22_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-D" "naming-lock CI enforces M22 runtime hardening runner" "${bool_has_m22_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-E" "naming-lock CI enforces M22 executed-slice convergence summary" "${bool_has_m22_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-F" "naming-lock CI enforces M22 transition handoff packet" "${bool_has_m22_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M22-G" "naming-lock CI enforces M22 closure report" "${bool_has_m22_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-A" "naming-lock CI enforces M23 kickoff brief" "${bool_has_m23_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-B" "naming-lock CI enforces M23 priority matrix" "${bool_has_m23_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-C" "naming-lock CI enforces M23 next-slice selector" "${bool_has_m23_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-D" "naming-lock CI enforces M23 runtime hardening runner" "${bool_has_m23_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-E" "naming-lock CI enforces M23 executed-slice convergence summary" "${bool_has_m23_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-F" "naming-lock CI enforces M23 transition handoff packet" "${bool_has_m23_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M23-G" "naming-lock CI enforces M23 closure report" "${bool_has_m23_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-A" "naming-lock CI enforces M24 kickoff brief" "${bool_has_m24_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-B" "naming-lock CI enforces M24 priority matrix" "${bool_has_m24_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-C" "naming-lock CI enforces M24 next-slice selector" "${bool_has_m24_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-D" "naming-lock CI enforces M24 runtime hardening runner" "${bool_has_m24_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-E" "naming-lock CI enforces M24 executed-slice convergence summary" "${bool_has_m24_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-F" "naming-lock CI enforces M24 transition handoff packet" "${bool_has_m24_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M24-G" "naming-lock CI enforces M24 closure report" "${bool_has_m24_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-A" "naming-lock CI enforces M25 kickoff brief" "${bool_has_m25_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-B" "naming-lock CI enforces M25 priority matrix" "${bool_has_m25_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-C" "naming-lock CI enforces M25 next-slice selector" "${bool_has_m25_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-D" "naming-lock CI enforces M25 runtime hardening runner" "${bool_has_m25_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-E" "naming-lock CI enforces M25 executed-slice convergence summary" "${bool_has_m25_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-F" "naming-lock CI enforces M25 transition handoff packet" "${bool_has_m25_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M25-G" "naming-lock CI enforces M25 closure report" "${bool_has_m25_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-A" "naming-lock CI enforces M26 kickoff brief" "${bool_has_m26_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-B" "naming-lock CI enforces M26 priority matrix" "${bool_has_m26_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-C" "naming-lock CI enforces M26 next-slice selector" "${bool_has_m26_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-D" "naming-lock CI enforces M26 runtime hardening runner" "${bool_has_m26_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-E" "naming-lock CI enforces M26 executed-slice convergence summary" "${bool_has_m26_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-F" "naming-lock CI enforces M26 transition handoff packet" "${bool_has_m26_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M26-G" "naming-lock CI enforces M26 closure report" "${bool_has_m26_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-A" "naming-lock CI enforces M27 kickoff brief" "${bool_has_m27_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-B" "naming-lock CI enforces M27 priority matrix" "${bool_has_m27_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-C" "naming-lock CI enforces M27 next-slice selector" "${bool_has_m27_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-D" "naming-lock CI enforces M27 runtime hardening runner" "${bool_has_m27_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-E" "naming-lock CI enforces M27 executed-slice convergence summary" "${bool_has_m27_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-F" "naming-lock CI enforces M27 transition handoff packet" "${bool_has_m27_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M27-G" "naming-lock CI enforces M27 closure report" "${bool_has_m27_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-A" "naming-lock CI enforces M28 kickoff brief" "${bool_has_m28_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-B" "naming-lock CI enforces M28 priority matrix" "${bool_has_m28_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-C" "naming-lock CI enforces M28 next-slice selector" "${bool_has_m28_slice_selector_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-D" "naming-lock CI enforces M28 runtime hardening runner" "${bool_has_m28_runtime_hardening_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-E" "naming-lock CI enforces M28 executed-slice convergence summary" "${bool_has_m28_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-F" "naming-lock CI enforces M28 transition handoff packet" "${bool_has_m28_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M28-G" "naming-lock CI enforces M28 closure report" "${bool_has_m28_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-A" "naming-lock CI enforces M29 kickoff brief" "${bool_has_m29_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-B" "naming-lock CI enforces M29 priority matrix" "${bool_has_m29_priority_matrix_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-C" "naming-lock CI enforces M29 runtime de-stub planner" "${bool_has_m29_runtime_destub_plan_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-D" "naming-lock CI enforces M29 runtime de-stub runner" "${bool_has_m29_runtime_destub_runner_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-E" "naming-lock CI enforces M29 executed-slice convergence summary" "${bool_has_m29_executed_slice_convergence_summary_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-F" "naming-lock CI enforces M29 transition handoff packet" "${bool_has_m29_transition_handoff_packet_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M29-G" "naming-lock CI enforces M29 closure report" "${bool_has_m29_closure_report_ci_guard}" "${naming_lock_workflow_path}"
emit_check "M30-A" "naming-lock CI enforces M30 kickoff brief" "${bool_has_m30_kickoff_brief_ci_guard}" "${naming_lock_workflow_path}"

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
