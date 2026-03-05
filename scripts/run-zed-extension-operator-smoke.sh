#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/run-zed-extension-operator-smoke.sh [--extensions-dir <path>] [--keep-temp]

Runs deterministic operator smoke flow:
  1) install local extension into clean profile dir,
  2) run plugin smoke checks,
  3) rollback and verify previous install is restored.

Options:
  --extensions-dir <path>  Use explicit extensions/installed directory.
  --keep-temp              Keep generated temporary profile directory.
  -h, --help               Show this help.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manage_script="${repo_root}/scripts/manage-zed-extension-local.sh"
plugin_smoke_script="${repo_root}/scripts/run-zed-plugin-smoke.sh"

extensions_dir=""
keep_temp=0
temp_root=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --extensions-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      extensions_dir="$2"
      shift 2
      ;;
    --extensions-dir=*)
      extensions_dir="${1#--extensions-dir=}"
      shift
      ;;
    --keep-temp)
      keep_temp=1
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

if [ ! -x "${manage_script}" ]; then
  echo "missing executable helper: ${manage_script}" >&2
  exit 1
fi

if [ ! -x "${plugin_smoke_script}" ]; then
  echo "missing executable smoke runner: ${plugin_smoke_script}" >&2
  exit 1
fi

if [ -z "${extensions_dir}" ]; then
  temp_root="$(mktemp -d)"
  extensions_dir="${temp_root}/extensions/installed"
fi

mkdir -p "${extensions_dir}"

cleanup() {
  if [ "${keep_temp}" -eq 1 ]; then
    if [ -n "${temp_root}" ]; then
      echo "kept temp root: ${temp_root}"
    fi
    return
  fi
  if [ -n "${temp_root}" ] && [ -d "${temp_root}" ]; then
    rm -rf "${temp_root}"
  fi
}
trap cleanup EXIT

# Seed a previous install so rollback can be asserted deterministically.
seed_dir="${extensions_dir}/untrusted"
mkdir -p "${seed_dir}"
printf '%s\n' "seed-before-install" > "${seed_dir}/SEED_MARKER.txt"

echo "[zed-operator-smoke] install"
"${manage_script}" install --mode copy --extensions-dir "${extensions_dir}"

if [ ! -f "${extensions_dir}/untrusted/extension.toml" ]; then
  echo "operator smoke failed: extension install is missing extension.toml" >&2
  exit 1
fi

echo "[zed-operator-smoke] run plugin smoke (--fast)"
"${plugin_smoke_script}" --fast

echo "[zed-operator-smoke] rollback"
"${manage_script}" rollback --extensions-dir "${extensions_dir}"

if [ ! -f "${extensions_dir}/untrusted/SEED_MARKER.txt" ]; then
  echo "operator smoke failed: rollback did not restore previous install marker" >&2
  exit 1
fi

echo "[zed-operator-smoke] passed"
