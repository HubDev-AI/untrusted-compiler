#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--matrix <path>] [--fail-on-warning]

Checks benchmark compare-matrix quality posture:
- endpoint entries must contain non-empty compared rows and a leader row aligned to endpoint
- leader row must be present in compared rows
- leader p99 must be present and > 0
- leader rssKb must be present and > 0
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
fail_count=0

printf '%-10s %-8s %-36s %s\n' "Endpoint" "Status" "Check" "Evidence"
printf '%-10s %-8s %-36s %s\n' "--------" "------" "------------------------------------" "--------"

while IFS=$'\t' read -r endpoint p99 rss_kb constant_rate target_rps actual_rps compared_count leader_in_compared leader_endpoint; do
  [ -z "${endpoint}" ] && continue

  if awk -v n="${compared_count}" 'BEGIN { exit !(n+0 > 0) }'; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "compared rows present" "compared=${compared_count}"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "FAIL" "missing compared rows" "compared=${compared_count}"
    fail_count=$((fail_count + 1))
  fi

  if [ "${leader_endpoint}" = "${endpoint}" ]; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader endpoint aligned" "leader.endpoint=${leader_endpoint}"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "FAIL" "leader endpoint mismatch" "leader.endpoint=${leader_endpoint:-<empty>}"
    fail_count=$((fail_count + 1))
  fi

  if [ "${leader_in_compared}" = "true" ]; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader row present in compared" "leaderInCompared=true"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "FAIL" "leader row missing in compared" "leaderInCompared=false"
    fail_count=$((fail_count + 1))
  fi

  p99_num="$(awk -v v="${p99}" 'BEGIN { if (match(v, /[0-9]+(\.[0-9]+)?/)) { print substr(v, RSTART, RLENGTH) } }')"
  if [ -n "${p99_num}" ] && awk -v n="${p99_num}" 'BEGIN { exit !(n+0 > 0) }'; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader p99 > 0" "p99=${p99}"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "WARN" "leader p99 missing/invalid" "p99=${p99:-<empty>}"
    warn_count=$((warn_count + 1))
  fi

  if awk -v n="${rss_kb}" 'BEGIN { exit !(n+0 > 0) }' >/dev/null 2>&1; then
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "PASS" "leader rssKb > 0" "rssKb=${rss_kb}"
    pass_count=$((pass_count + 1))
  else
    printf '%-10s %-8s %-36s %s\n' "${endpoint}" "WARN" "leader rssKb missing/invalid" "rssKb=${rss_kb:-<empty>}"
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
    def normalize_row:
      . as $row
      | {
          impl: ($row.impl // ""),
          endpoint: ($row.endpoint // ""),
          targetRps: ($row.targetRps // 0),
          requestsPerSec: ($row.requestsPerSec // 0),
          p99: ($row.p99 // ""),
          rssKb: ($row.rssKb // null),
          loadGenerator: (if ($row | has("loadGenerator")) then $row.loadGenerator else "wrk2" end),
          constantRate: (if ($row | has("constantRate")) then $row.constantRate else true end)
        };
    def row_eq(a; b):
      a.impl == b.impl
      and a.endpoint == b.endpoint
      and a.targetRps == b.targetRps
      and a.requestsPerSec == b.requestsPerSec
      and a.p99 == b.p99
      and a.rssKb == b.rssKb
      and a.loadGenerator == b.loadGenerator
      and a.constantRate == b.constantRate;
    .endpoints[]
    | .endpoint as $ep
    | (.leader // {} | normalize_row) as $leader
    | (.compared // [] | map(normalize_row)) as $compared
    | (
        ($compared | type == "array")
        and any($compared[]; row_eq(.; $leader))
      ) as $leader_in_compared
    | [
        $ep,
        ($leader.p99 // ""),
        (if ($leader.rssKb // null) == null then "null" else ($leader.rssKb | tostring) end),
        (if ($leader | has("constantRate")) then $leader.constantRate else true end | tostring),
        ($leader.targetRps // 0 | tostring),
        ($leader.requestsPerSec // 0 | tostring),
        ($compared | length | tostring),
        ($leader_in_compared | tostring),
        ($leader.endpoint // "")
      ]
    | @tsv
  ' "${matrix_path}"
)

echo
if [ "${fail_count}" -gt 0 ]; then
  echo "overall: FAIL (${fail_count} fail check(s), ${warn_count} warning(s), ${pass_count} pass check(s))"
  exit 2
fi

if [ "${warn_count}" -eq 0 ]; then
  echo "overall: PASS (quality checks passed)"
  exit 0
fi

echo "overall: WARN (${warn_count} warning(s), ${pass_count} pass check(s))"
if [ "${fail_on_warning}" = "true" ]; then
  exit 1
fi
