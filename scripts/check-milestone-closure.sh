#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>] [--matrix <path>] [--trend-note <path>] [--fail-on-pending]

Checks strict closure evidence for milestone status gates (M9/M10/M13).
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_path=""
trend_note_path=""
fail_on_pending="false"

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

bool_has_release_gate=0
bool_has_release_gate_ci=0
bool_has_release_verifier_chain=0
bool_has_release_gate_closure_enforcement=0
bool_has_cross_impl_matrix=0
bool_has_cross_impl_matrix_contract=0
bool_has_live_trend_entry=0
bool_has_trend_workflow_guards=0
bool_has_trend_workflow_artifact_upload=0

[ -f "${repo_root}/scripts/release-alpha-gate.sh" ] && bool_has_release_gate=1
[ -f "${repo_root}/.github/workflows/alpha-release-gate.yml" ] && bool_has_release_gate_ci=1
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

pending_count=0

emit_check() {
  local code="$1"
  local label="$2"
  local pass="$3"
  local evidence="$4"
  local status
  status="$(status_for "${pass}")"
  if [ "${pass}" -eq 0 ]; then
    pending_count=$((pending_count + 1))
  fi
  printf '%-6s %-8s %-64s %s\n' "${code}" "${status}" "${label}" "${evidence}"
}

echo "Milestone Closure Audit"
echo "repo: ${repo_root}"
echo
printf '%-6s %-8s %-64s %s\n' "Gate" "Status" "Check" "Evidence"
printf '%-6s %-8s %-64s %s\n' "-----" "--------" "----------------------------------------------------------------" "--------"

emit_check "M9-A" "release gate script exists" "${bool_has_release_gate}" "scripts/release-alpha-gate.sh"
emit_check "M9-B" "release gate workflow exists" "${bool_has_release_gate_ci}" ".github/workflows/alpha-release-gate.yml"
emit_check "M9-C" "promotion verifier/manifest chain exists" "${bool_has_release_verifier_chain}" "scripts/verify-release-promotion-inputs.sh + publish-manifest scripts"
emit_check "M9-D" "release gate enforces strict milestone closure" "${bool_has_release_gate_closure_enforcement}" "scripts/release-alpha-gate.sh"
emit_check "M10-A" "cross-impl matrix includes sec4/go/node/rust for each endpoint" "${bool_has_cross_impl_matrix}" "${matrix_path}"
emit_check "M10-B" "cross-impl matrix row contract is aligned" "${bool_has_cross_impl_matrix_contract}" "${matrix_path}"
emit_check "M13-A" "trend note contains at least one live Trend Entry block" "${bool_has_live_trend_entry}" "${trend_note_path}"
emit_check "M13-B" "benchmark trend workflow has strict quality + regression guards" "${bool_has_trend_workflow_guards}" "${trend_workflow_path}"
emit_check "M13-C" "benchmark trend workflow uploads trend artifacts" "${bool_has_trend_workflow_artifact_upload}" "${trend_workflow_path}"

echo
if [ "${pending_count}" -eq 0 ]; then
  echo "overall: PASS (all tracked closure checks satisfied)"
  exit 0
fi

echo "overall: PENDING (${pending_count} check(s) not yet satisfied)"
if [ "${fail_on_pending}" = "true" ]; then
  exit 1
fi
