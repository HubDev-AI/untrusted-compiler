#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--impls ailang,node,go,rust,c] [--dry-run-only]" >&2
}

impls_csv="ailang,node,go,rust,c"
dry_run_only="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      impls_csv="$2"
      shift 2
      ;;
    --impls=*)
      impls_csv="${1#--impls=}"
      shift
      ;;
    --dry-run-only)
      dry_run_only="true"
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

is_supported_impl() {
  case "$1" in
    ailang|node|go|rust|c) return 0 ;;
    *) return 1 ;;
  esac
}

check_cmd() {
  local cmd="$1"
  local label="$2"
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "OK       ${label} (${cmd})"
    return 0
  fi
  echo "MISSING  ${label} (${cmd})"
  return 1
}

missing=0

check_cmd "curl" "HTTP probing" || missing=1
check_cmd "jq" "JSON processing" || missing=1

if [ "$dry_run_only" != "true" ]; then
  check_cmd "wrk2" "load generator" || missing=1
fi

IFS=',' read -r -a impls <<< "$impls_csv"
for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue

  if ! is_supported_impl "$impl"; then
    echo "MISSING  unsupported implementation '${impl}'" >&2
    exit 2
  fi

  case "$impl" in
    ailang)
      check_cmd "cargo" "AILang compiler runner" || missing=1
      check_cmd "cc" "C compiler for AILang runtime link" || missing=1
      ;;
    node)
      check_cmd "node" "Node benchmark service" || missing=1
      ;;
    go)
      check_cmd "go" "Go benchmark service" || missing=1
      ;;
    rust)
      check_cmd "cargo" "Rust benchmark service" || missing=1
      ;;
    c)
      check_cmd "cc" "C benchmark service" || missing=1
      ;;
  esac
done

if [ "$missing" -ne 0 ]; then
  echo "preflight failed: missing required tooling" >&2
  exit 1
fi

echo "preflight passed"
