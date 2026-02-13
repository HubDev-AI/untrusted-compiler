#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo owner/repo] [--workflow benchmark-cross-impl-evidence.yml] [--artifact-name benchmark-cross-impl-evidence] [--out-dir <path>] [--matrix <path>] [--target <path>] [--dry-run]

Fetches latest cross-impl benchmark artifact (unless --matrix is provided), validates compare-matrix coverage, and writes target matrix path.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

repo_slug="HubDev-AI/untrusted-compiler"
workflow_name="benchmark-cross-impl-evidence.yml"
artifact_name="benchmark-cross-impl-evidence"
out_dir="${repo_root}/benchmark-suite/results/cross-impl-download"
matrix_path=""
target_path="${repo_root}/benchmark-suite/results/summaries/compare-matrix.json"
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
    --workflow)
      workflow_name="$2"
      shift 2
      ;;
    --workflow=*)
      workflow_name="${1#--workflow=}"
      shift
      ;;
    --artifact-name)
      artifact_name="$2"
      shift 2
      ;;
    --artifact-name=*)
      artifact_name="${1#--artifact-name=}"
      shift
      ;;
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    --out-dir=*)
      out_dir="${1#--out-dir=}"
      shift
      ;;
    --matrix)
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --target)
      target_path="$2"
      shift 2
      ;;
    --target=*)
      target_path="${1#--target=}"
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

fetch_cmd=(gh run list --repo "${repo_slug}" --workflow "${workflow_name}" --json databaseId,status,conclusion --limit 20)
download_cmd=(gh run download "<run_id>" --repo "${repo_slug}" --name "${artifact_name}" --dir "${out_dir}")

if [ "${dry_run}" = "true" ]; then
  if [ -z "${matrix_path}" ]; then
    echo "run: ${fetch_cmd[*]}"
    echo "run: ${download_cmd[*]}"
    echo "run: find ${out_dir} -type f -name compare-matrix.json"
  fi
  echo "run: validate matrix includes impls sec4,go,node,rust"
  echo "run: copy matrix to ${target_path}"
  exit 0
fi

if [ -z "${matrix_path}" ]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI is required" >&2
    exit 2
  fi
  if ! gh auth status >/dev/null 2>&1; then
    echo "gh auth is required; run: gh auth login -h github.com" >&2
    exit 2
  fi
  runs_json="$("${fetch_cmd[@]}")"
  run_id="$(jq -r '[.[] | select(.status == "completed" and .conclusion == "success")][0].databaseId // empty' <<<"${runs_json}")"
  if [ -z "${run_id}" ]; then
    echo "no successful completed workflow runs found for ${workflow_name}" >&2
    exit 1
  fi
  mkdir -p "${out_dir}"
  gh run download "${run_id}" --repo "${repo_slug}" --name "${artifact_name}" --dir "${out_dir}"
  matrix_path="$(find "${out_dir}" -type f -name compare-matrix.json | sort | tail -n 1)"
  if [ -z "${matrix_path}" ]; then
    echo "compare-matrix.json not found under ${out_dir}" >&2
    exit 1
  fi
fi

if [ ! -f "${matrix_path}" ]; then
  echo "matrix file not found: ${matrix_path}" >&2
  exit 2
fi

if ! jq -e '
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
' "${matrix_path}" >/dev/null; then
  echo "matrix does not include required impl set (sec4/go/node/rust): ${matrix_path}" >&2
  exit 1
fi

mkdir -p "$(dirname "${target_path}")"
cp "${matrix_path}" "${target_path}"
echo "updated cross-impl matrix evidence: ${target_path}"
