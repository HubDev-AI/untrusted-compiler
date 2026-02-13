#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--matrix <path>] [--fail-on-warning]

Checks benchmark compare-matrix quality posture:
- leader p99 must be present and > 0
- leader constantRate=false is flagged as WARN (non-constant-rate evidence)

USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_path="${repo_root}/benchmark-suite/results/summaries/compare-matrix.json"
fail_on_warning="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --matrix)
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --fail-on-warning)
      fail_on_warning="true"
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

if [ ! -f "${matrix_path}" ]; then
  echo "compare matrix not found: ${matrix_path}" >&2
  exit 2
fi

if [ "$(jq '.endpoints | type == "array" and length > 0' "${matrix_path}")" != "true" ]; then
  echo "compare matrix has no endpoints: ${matrix_path}" >&2
  exit 2
fi

warn_count=0
pass_count=0

printf '%-10s %-8s %-36s %s\n' "Endpoint" "Status" "Check" "Evidence"
printf '%-10s %-8s %-36s %s\n' "--------" "------" "------------------------------------" "--------"

while IFS=$'\t' read -r endpoint p99 constant_rate target_rps actual_rps; do
  [ -z "${endpoint}" ] && continue

  p99_num="$(awk -v v="${p99}" 'BEGIN { if (match(v, /[0-9]+(\.[0-9]+)?/)) { print substr(v, RSTART, RLENGTH) } }')"
  if [ -n "${p99_num}" ] && awk -v n="${p99_num}" 'BEGIN { exit !(n+0 > 0) }'; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader p99 > 0" "p99=${p99}"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "WARN" "leader p99 missing/invalid" "p99=${p99:-<empty>}"
    warn_count=$((warn_count + 1))
  fi

  if [ "${constant_rate}" = "false" ]; then
    coverage="n/a"
    if awk -v t="${target_rps}" 'BEGIN { exit !(t+0 > 0) }'; then
      coverage="$(awk -v t="${target_rps}" -v a="${actual_rps}" 'BEGIN { printf "%.2f", (a/t)*100 }')"
      coverage="${coverage}%"
    fi
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "WARN" "leader non-constant-rate run" "constantRate=false coverage=${coverage}"
    warn_count=$((warn_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader constant-rate run" "constantRate=${constant_rate}"
    pass_count=$((pass_count + 1))
  fi
done < <(
  jq -r '
    .endpoints[]
    | .endpoint as $ep
    | (.leader // {}) as $leader
    | [
        $ep,
        ($leader.p99 // ""),
        (if ($leader | has("constantRate")) then $leader.constantRate else true end | tostring),
        ($leader.targetRps // 0 | tostring),
        ($leader.requestsPerSec // 0 | tostring)
      ]
    | @tsv
  ' "${matrix_path}"
)

echo
if [ "${warn_count}" -eq 0 ]; then
  echo "overall: PASS (quality checks passed)"
  exit 0
fi

echo "overall: WARN (${warn_count} warning(s), ${pass_count} pass check(s))"
if [ "${fail_on_warning}" = "true" ]; then
  exit 1
fi
