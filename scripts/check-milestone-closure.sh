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
bool_has_cross_impl_matrix=0
bool_has_live_trend_entry=0

[ -f "${repo_root}/scripts/release-alpha-gate.sh" ] && bool_has_release_gate=1
[ -f "${repo_root}/.github/workflows/alpha-release-gate.yml" ] && bool_has_release_gate_ci=1
[ -f "${repo_root}/scripts/verify-release-promotion-inputs.sh" ] \
  && [ -f "${repo_root}/scripts/generate-release-publish-manifest.sh" ] \
  && [ -f "${repo_root}/scripts/verify-release-publish-manifest.sh" ] \
  && bool_has_release_verifier_chain=1

if [ -f "${matrix_path}" ]; then
  if jq -e '
    .endpoints as $eps
    | ($eps | type == "array")
    and ($eps | length > 0)
    and (
      [ $eps[]?.compared[]?.impl ] | unique | sort
      | (index("sec4") != null)
      and (index("go") != null)
      and (index("node") != null)
      and (index("rust") != null)
    )
  ' "${matrix_path}" >/dev/null 2>&1; then
    bool_has_cross_impl_matrix=1
  fi
fi

if [ -f "${trend_note_path}" ]; then
  if rg -q '^## Trend Entry \([0-9]{4}-[0-9]{2}-[0-9]{2}\)$' "${trend_note_path}"; then
    bool_has_live_trend_entry=1
  fi
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
emit_check "M10-A" "cross-impl compare matrix evidence includes sec4/go/node/rust" "${bool_has_cross_impl_matrix}" "${matrix_path}"
emit_check "M13-A" "trend note contains at least one live Trend Entry block" "${bool_has_live_trend_entry}" "${trend_note_path}"

echo
if [ "${pending_count}" -eq 0 ]; then
  echo "overall: PASS (all tracked closure checks satisfied)"
  exit 0
fi

echo "overall: PENDING (${pending_count} check(s) not yet satisfied)"
if [ "${fail_on_pending}" = "true" ]; then
  exit 1
fi
