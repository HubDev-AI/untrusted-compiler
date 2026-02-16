#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--repo-root <path>]

Fails when repository files contain forbidden absolute local paths.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

forbidden_token="/Users/""vladimirtrifonov""/src/ai/"

if [ ! -d "${repo_root}" ]; then
  echo "repo root does not exist: ${repo_root}" >&2
  exit 1
fi

matches="$(
  if command -v rg >/dev/null 2>&1; then
    rg -n --fixed-strings --hidden --glob '!**/.git/**' --glob '!**/target/**' --glob '!**/build/**' --glob '!**/node_modules/**' --glob '!**/.DS_Store' "${forbidden_token}" "${repo_root}" || true
  else
    grep -R -n -F \
      --exclude-dir='.git' \
      --exclude-dir='target' \
      --exclude-dir='build' \
      --exclude-dir='node_modules' \
      --exclude='.DS_Store' \
      -- "${forbidden_token}" "${repo_root}" || true
  fi
)"

if [ -n "${matches}" ]; then
  echo "local path leak detected (${forbidden_token})" >&2
  echo "${matches}" >&2
  exit 1
fi

echo "local path leak check passed"
