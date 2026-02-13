#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo owner/repo] [--matrix <path>] [--entry <path>] [--target-matrix <path>] [--trend-note <path>] [--m10-out-dir <path>] [--m13-out-dir <path>] [--dry-run]

Runs M10 + M13 evidence refresh in sequence:
1) update cross-impl compare matrix from CI artifact (or --matrix)
2) update trend note chapter from CI artifact (or --entry)
3) run strict closure audit with fail-on-pending
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

repo_slug="HubDev-AI/untrusted-compiler"
matrix_source=""
entry_source=""
target_matrix="${repo_root}/benchmark-suite/results/summaries/compare-matrix.json"
trend_note="${repo_root}/docs/book/322-m13-first-trend-run-results-note.md"
m10_out_dir="${repo_root}/benchmark-suite/results/cross-impl-download"
m13_out_dir="${repo_root}/benchmark-suite/results/trend-download"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      repo_slug="$2"
      shift 2
      ;;
    --repo=*)
      repo_slug="${1#--repo=}"
      shift
      ;;
    --matrix)
      matrix_source="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_source="${1#--matrix=}"
      shift
      ;;
    --entry)
      entry_source="$2"
      shift 2
      ;;
    --entry=*)
      entry_source="${1#--entry=}"
      shift
      ;;
    --target-matrix)
      target_matrix="$2"
      shift 2
      ;;
    --target-matrix=*)
      target_matrix="${1#--target-matrix=}"
      shift
      ;;
    --trend-note)
      trend_note="$2"
      shift 2
      ;;
    --trend-note=*)
      trend_note="${1#--trend-note=}"
      shift
      ;;
    --m10-out-dir)
      m10_out_dir="$2"
      shift 2
      ;;
    --m10-out-dir=*)
      m10_out_dir="${1#--m10-out-dir=}"
      shift
      ;;
    --m13-out-dir)
      m13_out_dir="$2"
      shift 2
      ;;
    --m13-out-dir=*)
      m13_out_dir="${1#--m13-out-dir=}"
      shift
      ;;
    --dry-run)
      dry_run="true"
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

m10_cmd=(
  "${repo_root}/benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh"
  --repo "${repo_slug}"
  --target "${target_matrix}"
)
if [ -n "${matrix_source}" ]; then
  m10_cmd+=(--matrix "${matrix_source}")
else
  m10_cmd+=(--out-dir "${m10_out_dir}")
fi

m13_cmd=(
  "${repo_root}/benchmark-suite/scripts/update_trend_note_from_ci.sh"
  --repo "${repo_slug}"
  --chapter "${trend_note}"
)
if [ -n "${entry_source}" ]; then
  m13_cmd+=(--entry "${entry_source}")
else
  m13_cmd+=(--out-dir "${m13_out_dir}")
fi

closure_cmd=(
  "${repo_root}/scripts/check-milestone-closure.sh"
  --matrix "${target_matrix}"
  --trend-note "${trend_note}"
  --fail-on-pending
)

if [ "${dry_run}" = "true" ]; then
  echo "run: ${m10_cmd[*]}"
  echo "run: ${m13_cmd[*]}"
  echo "run: ${closure_cmd[*]}"
  exit 0
fi

"${m10_cmd[@]}"
"${m13_cmd[@]}"
"${closure_cmd[@]}"

echo "closure evidence refreshed and validated"
