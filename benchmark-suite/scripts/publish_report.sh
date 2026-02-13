#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 <compare_matrix.json> <out_report.md> [sec_audit.json]" >&2
  exit 2
fi

matrix_path="$1"
out_path="$2"
sec_audit_path="${3:-}"

if [ ! -f "$matrix_path" ]; then
  echo "compare matrix file not found: ${matrix_path}" >&2
  exit 2
fi

if [ "$(jq '.endpoints | length' "$matrix_path")" -eq 0 ]; then
  echo "compare matrix has no endpoints: ${matrix_path}" >&2
  exit 2
fi

if [ -n "$sec_audit_path" ] && [ ! -f "$sec_audit_path" ]; then
  echo "sec audit file not found: ${sec_audit_path}" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"
now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
  echo "# Benchmark Comparative Report (v0.1)"
  echo
  echo "- Generated UTC: ${now_utc}"
  echo "- Matrix source: ${matrix_path}"
  if [ -n "$sec_audit_path" ]; then
    echo "- Security source: ${sec_audit_path}"
  fi
  echo

  echo "## Endpoint Leaders"
  echo
  echo "| Endpoint | Leader | Requests/sec | p99 |"
  echo "|---|---|---:|---:|"
  jq -r '.endpoints[] | [.endpoint, (.leader.impl // "n/a"), ((.leader.requestsPerSec // 0)|tostring), (.leader.p99 // "")] | @tsv' "$matrix_path" \
    | while IFS=$'\t' read -r endpoint leader reqps p99; do
        printf "| %s | %s | %s | %s |\n" "$endpoint" "$leader" "$reqps" "$p99"
      done
  echo

  echo "## Endpoint Rankings"
  echo
  jq -r '.endpoints[] | .endpoint' "$matrix_path" | while IFS= read -r endpoint; do
    [ -z "$endpoint" ] && continue
    echo "### ${endpoint}"
    rank=1
    jq -r --arg endpoint "$endpoint" '.endpoints[] | select(.endpoint == $endpoint) | .compared[] | [.impl, (.requestsPerSec|tostring), (.p99 // "")] | @tsv' "$matrix_path" \
      | while IFS=$'\t' read -r impl reqps p99; do
          printf "%d. %s - req/s: %s, p99: %s\n" "$rank" "$impl" "$reqps" "$p99"
          rank=$((rank + 1))
        done
    echo
  done

  echo "## Tail Latency Signals"
  echo
  jq -r '
    def p99num:
      ((.p99 // "") | tostring | capture("(?<n>[0-9]+(\\.[0-9]+)?)")?.n // "0" | tonumber);
    .endpoints[]
    | .endpoint as $ep
    | (.compared | map(p99num)) as $vals
    | {
        endpoint: $ep,
        p99Min: ($vals | min),
        p99Max: ($vals | max),
        ratio: (if (($vals | min) > 0) then (($vals | max) / ($vals | min)) else null end)
      }
    | . as $row
    | "- \($row.endpoint): p99 min=\($row.p99Min)ms, max=\($row.p99Max)ms, spread="
      + (if $row.ratio == null then "n/a" else (((($row.ratio * 100) | round) / 100) | tostring) + "x" end)
  ' "$matrix_path"
  echo

  echo "## Security Posture"
  echo
  if [ -n "$sec_audit_path" ]; then
    policy_name="$(jq -r '.policy.name // "unknown"' "$sec_audit_path")"
    policy_hash="$(jq -r '.policy.hash // "unknown"' "$sec_audit_path")"
    highest_severity="$(jq -r '.summary.highestSeverity // "unknown"' "$sec_audit_path")"
    risk_score="$(jq -r '.summary.riskScore // "unknown"' "$sec_audit_path")"
    findings_count="$(jq -r '.findings | length' "$sec_audit_path")"

    echo "- Policy: ${policy_name} (${policy_hash})"
    echo "- Findings: ${findings_count}"
    echo "- Highest severity: ${highest_severity}"
    echo "- Risk score: ${risk_score}"

    if [ "$findings_count" -gt 0 ]; then
      echo
      echo "Top findings:"
      jq -r '.findings[] | "- [\(.severity)] \(.id): \(.suggestion)"' "$sec_audit_path" | head -n 5
    fi
  else
    echo "- No sec.audit artifact provided."
  fi
} > "$out_path"

echo "wrote $out_path"
