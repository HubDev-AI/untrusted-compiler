#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--impls sec4,sec4-lasm,node,go,rust,c] [--dry-run-only]" >&2
}

impls_csv="sec4,sec4-lasm,node,go,rust,c"
dry_run_only="false"
require_wrk2="${BENCH_REQUIRE_WRK2:-0}"

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
    sec4|sec4-lasm|node|go|rust|c) return 0 ;;
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

is_truthy() {
  case "$1" in
    1|true|TRUE|yes|YES|on|ON)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

check_load_generator() {
  if command -v wrk2 >/dev/null 2>&1; then
    echo "OK       load generator (wrk2)"
    return 0
  fi
  if is_truthy "$require_wrk2"; then
    echo "MISSING  load generator (wrk2 required)"
    return 1
  fi
  if command -v wrk >/dev/null 2>&1; then
    echo "OK       load generator (wrk fallback) (wrk)"
    return 0
  fi
  echo "MISSING  load generator (wrk2/wrk)"
  return 1
}

missing=0

check_cmd "curl" "HTTP probing" || missing=1
check_cmd "jq" "JSON processing" || missing=1

if [ "$dry_run_only" != "true" ]; then
  check_load_generator || missing=1
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
    sec4)
      check_cmd "cargo" "Untrusted<T> compiler runner" || missing=1
      check_cmd "cc" "C compiler for Untrusted<T> runtime link" || missing=1
      ;;
    sec4-lasm)
      check_cmd "cargo" "Untrusted<T> compiler runner" || missing=1
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
