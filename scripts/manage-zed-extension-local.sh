#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/manage-zed-extension-local.sh [install|update|rollback|status] [options]

Commands:
  install   Install the repository Zed extension into local Zed extensions dir.
  update    Alias of install (backup existing install, then install current repo state).
  rollback  Restore the most recent backup.
  status    Print resolved paths and current install/backup state.

Options:
  --extensions-dir <path>  Override target Zed extensions/installed directory.
  --mode copy|symlink      Install mode (default: copy).
  --backup-root <path>     Override backup root directory.
  -h, --help               Show this help.

Notes:
  - Default target directory is auto-detected by platform:
      macOS: ~/Library/Application Support/Zed/extensions/installed
      Linux: ~/.local/share/zed/extensions/installed
  - Existing installs are moved to timestamped backups before install/update.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${repo_root}/zed-extension"
manifest_path="${source_dir}/extension.toml"

command="install"
mode="copy"
extensions_dir=""
backup_root=""

if [ "${1:-}" = "install" ] || [ "${1:-}" = "update" ] || [ "${1:-}" = "rollback" ] || [ "${1:-}" = "status" ]; then
  command="$1"
  shift
fi

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
    --mode)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      mode="$2"
      shift 2
      ;;
    --mode=*)
      mode="${1#--mode=}"
      shift
      ;;
    --backup-root)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      backup_root="$2"
      shift 2
      ;;
    --backup-root=*)
      backup_root="${1#--backup-root=}"
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

if [ "${mode}" != "copy" ] && [ "${mode}" != "symlink" ]; then
  echo "invalid --mode '${mode}' (expected copy or symlink)" >&2
  exit 2
fi

if [ ! -f "${manifest_path}" ]; then
  echo "missing extension manifest: ${manifest_path}" >&2
  exit 1
fi

extension_id="$(awk -F'"' '/^id[[:space:]]*=[[:space:]]*"/ { print $2; exit }' "${manifest_path}")"
if [ -z "${extension_id}" ]; then
  echo "failed to read extension id from ${manifest_path}" >&2
  exit 1
fi

default_extensions_dir() {
  local candidates=()
  case "$(uname -s)" in
    Darwin)
      candidates=("${HOME}/Library/Application Support/Zed/extensions/installed")
      ;;
    Linux)
      candidates=(
        "${HOME}/.local/share/zed/extensions/installed"
        "${HOME}/.local/share/Zed/extensions/installed"
      )
      ;;
    *)
      candidates=(
        "${HOME}/.local/share/zed/extensions/installed"
        "${HOME}/Library/Application Support/Zed/extensions/installed"
      )
      ;;
  esac

  local candidate
  for candidate in "${candidates[@]}"; do
    if [ -d "${candidate}" ]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  printf '%s\n' "${candidates[0]}"
}

if [ -z "${extensions_dir}" ]; then
  extensions_dir="$(default_extensions_dir)"
fi

target_dir="${extensions_dir}/${extension_id}"

if [ -z "${backup_root}" ]; then
  backup_root="${extensions_dir}/.sec4-backups/${extension_id}"
fi

timestamp() {
  date -u +"%Y%m%dT%H%M%SZ"
}

latest_backup() {
  if [ ! -d "${backup_root}" ]; then
    return 1
  fi
  ls -1dt "${backup_root}"/* 2>/dev/null | head -n 1
}

print_status() {
  echo "repo_root=${repo_root}"
  echo "source_dir=${source_dir}"
  echo "manifest_path=${manifest_path}"
  echo "extension_id=${extension_id}"
  echo "extensions_dir=${extensions_dir}"
  echo "target_dir=${target_dir}"
  echo "backup_root=${backup_root}"
  echo "mode=${mode}"
  if [ -L "${target_dir}" ]; then
    echo "install_state=symlink"
    echo "install_target=$(readlink "${target_dir}")"
  elif [ -d "${target_dir}" ]; then
    echo "install_state=directory"
  else
    echo "install_state=missing"
  fi

  if latest="$(latest_backup 2>/dev/null || true)"; then
    if [ -n "${latest}" ]; then
      echo "latest_backup=${latest}"
    else
      echo "latest_backup=none"
    fi
  else
    echo "latest_backup=none"
  fi
}

backup_existing_install() {
  if [ ! -e "${target_dir}" ]; then
    return 0
  fi
  local backup_dir="${backup_root}/$(timestamp)"
  mkdir -p "${backup_root}"
  mv "${target_dir}" "${backup_dir}"
  echo "moved existing install to backup: ${backup_dir}"
}

install_or_update() {
  mkdir -p "${extensions_dir}"
  backup_existing_install

  if [ "${mode}" = "copy" ]; then
    cp -R "${source_dir}" "${target_dir}"
    echo "installed extension via copy: ${target_dir}"
  else
    ln -s "${source_dir}" "${target_dir}"
    echo "installed extension via symlink: ${target_dir} -> ${source_dir}"
  fi
}

rollback_install() {
  local restore_from
  restore_from="$(latest_backup 2>/dev/null || true)"
  if [ -z "${restore_from}" ]; then
    echo "no backup found in ${backup_root}" >&2
    exit 1
  fi

  if [ -e "${target_dir}" ]; then
    local current_backup="${backup_root}/current-$(timestamp)"
    mkdir -p "${backup_root}"
    mv "${target_dir}" "${current_backup}"
    echo "moved current install to backup: ${current_backup}"
  fi

  mv "${restore_from}" "${target_dir}"
  echo "restored backup: ${restore_from} -> ${target_dir}"
}

case "${command}" in
  install)
    install_or_update
    ;;
  update)
    install_or_update
    ;;
  rollback)
    rollback_install
    ;;
  status)
    print_status
    ;;
  *)
    echo "unsupported command: ${command}" >&2
    usage
    exit 2
    ;;
esac
