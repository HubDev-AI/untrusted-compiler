#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--audit-file <path>] [--explain-file <path>]

Fails if any sec4-core audit finding IDs are not mapped in sec4 explain_topic.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
audit_file="${repo_root}/compiler/sec4-core/src/audit.rs"
explain_file="${repo_root}/compiler/sec4-cli/src/main.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --audit-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      audit_file="$2"
      shift 2
      ;;
    --audit-file=*)
      audit_file="${1#--audit-file=}"
      shift
      ;;
    --explain-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      explain_file="$2"
      shift 2
      ;;
    --explain-file=*)
      explain_file="${1#--explain-file=}"
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

if [ ! -f "${audit_file}" ]; then
  echo "audit file not found: ${audit_file}" >&2
  exit 2
fi

if [ ! -f "${explain_file}" ]; then
  echo "explain file not found: ${explain_file}" >&2
  exit 2
fi

audit_ids="$(
  awk '
    /finding\(/ {
      finding_open = 1
    }
    finding_open {
      if (match($0, /"([A-Z0-9_]+)"/)) {
        value = substr($0, RSTART + 1, RLENGTH - 2)
        print value
        finding_open = 0
      }
    }
  ' "${audit_file}" | sort -u
)"

if [ -z "${audit_ids}" ]; then
  echo "no audit finding IDs detected in ${audit_file}" >&2
  exit 2
fi

explain_ids="$(
  rg -o '"[A-Z0-9_]+"\s*=>\s*\{' "${explain_file}" \
    | sed -E 's/"([A-Z0-9_]+)".*/\1/' \
    | sort -u
)"

if [ -z "${explain_ids}" ]; then
  echo "no explain mapping IDs detected in ${explain_file}" >&2
  exit 2
fi

missing_ids="$(comm -23 <(printf '%s\n' "${audit_ids}") <(printf '%s\n' "${explain_ids}"))"
if [ -n "${missing_ids}" ]; then
  echo "missing sec4 explain mappings for audit finding IDs:" >&2
  printf '  - %s\n' ${missing_ids} >&2
  exit 1
fi

covered_count="$(printf '%s\n' "${audit_ids}" | wc -l | tr -d ' ')"
echo "ok: sec4 explain covers all audit finding IDs (${covered_count})"
