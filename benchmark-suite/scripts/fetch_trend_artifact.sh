#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo owner/repo] [--workflow benchmark-trend.yml] [--artifact-name benchmark-trend-node-ping-decode] [--out-dir <path>] [--dry-run]

Fetches the latest successful benchmark trend artifact from GitHub Actions.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

repo_slug=""
workflow_name="benchmark-trend.yml"
artifact_name="benchmark-trend-node-ping-decode"
out_dir="${repo_root}/benchmark-suite/results/trend-download"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      repo_slug="$2"
      shift 2
      ;;
    --repo=*)
      repo_slug="${1#--repo=}"
      shift
      ;;
    --workflow)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      workflow_name="$2"
      shift 2
      ;;
    --workflow=*)
      workflow_name="${1#--workflow=}"
      shift
      ;;
    --artifact-name)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      artifact_name="$2"
      shift 2
      ;;
    --artifact-name=*)
      artifact_name="${1#--artifact-name=}"
      shift
      ;;
    --out-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_dir="$2"
      shift 2
      ;;
    --out-dir=*)
      out_dir="${1#--out-dir=}"
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

if [ -z "${repo_slug}" ]; then
  origin_url="$(git -C "${repo_root}" remote get-url origin 2>/dev/null || true)"
  repo_slug="$(sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##' <<<"${origin_url}")"
fi

if [ -z "${repo_slug}" ]; then
  echo "unable to infer repository slug; pass --repo owner/repo" >&2
  exit 2
fi

list_cmd=(gh run list --repo "${repo_slug}" --workflow "${workflow_name}" --json databaseId,status,conclusion --limit 20)

if [ "${dry_run}" = "true" ]; then
  echo "run: ${list_cmd[*]}"
  echo "run: gh run download <run_id> --repo ${repo_slug} --name ${artifact_name} --dir ${out_dir}"
  exit 0
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required" >&2
  exit 2
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "gh auth is required; run: gh auth login -h github.com" >&2
  exit 2
fi

runs_json="$("${list_cmd[@]}")"
run_id="$(jq -r '[.[] | select(.status == "completed" and .conclusion == "success")][0].databaseId // empty' <<<"${runs_json}")"
if [ -z "${run_id}" ]; then
  echo "no successful completed workflow runs found for ${workflow_name}" >&2
  exit 1
fi

mkdir -p "${out_dir}"
gh run download "${run_id}" --repo "${repo_slug}" --name "${artifact_name}" --dir "${out_dir}"

echo "downloaded artifact '${artifact_name}' from run ${run_id} into ${out_dir}"
