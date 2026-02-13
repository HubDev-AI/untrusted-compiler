#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--dry-run] [--impls ailang,node,go,rust,c]" >&2
}

dry_run="false"
impls_csv="ailang,node,go,rust,c"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
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

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

is_supported_impl() {
  case "$1" in
    ailang|node|go|rust|c) return 0 ;;
    *) return 1 ;;
  esac
}

IFS=',' read -r -a impls <<< "$impls_csv"
for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue
  if ! is_supported_impl "$impl"; then
    echo "unsupported implementation: ${impl}" >&2
    exit 2
  fi

done

for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue

  smoke_script="${root_dir}/services/${impl}/smoke.sh"
  if [ ! -x "$smoke_script" ]; then
    echo "missing executable smoke script for ${impl}: ${smoke_script}" >&2
    exit 1
  fi

  if [ "$dry_run" = "true" ]; then
    echo "run: ${smoke_script}"
    continue
  fi

  echo "=== smoke impl=${impl} ==="
  "$smoke_script"
done

echo "service contract smoke checks passed"
