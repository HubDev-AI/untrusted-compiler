#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 4 ]; then
  echo "usage: $0 <compare_matrix.json> <out_report.md> [sec_audit.json] [analysis.json]" >&2
  exit 2
fi

matrix_path="$1"
out_path="$2"
sec_audit_path="${3:-}"
analysis_path="${4:-}"

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

if [ -n "$analysis_path" ] && [ ! -f "$analysis_path" ]; then
  echo "analysis file not found: ${analysis_path}" >&2
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
  if [ -n "$analysis_path" ]; then
    echo "- Analysis source: ${analysis_path}"
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
  if [ -n "$analysis_path" ]; then
    jq -r '.endpoints[] | .metrics as $m | "- \(.endpoint): p99 min=\($m.p99MinMs)ms, max=\($m.p99MaxMs)ms, spread=\(if $m.p99SpreadX == null then "n/a" else (((($m.p99SpreadX * 100) | round) / 100) | tostring) + "x" end)"' "$analysis_path"
  else
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
  fi
  echo

  echo "## Matrix Analysis"
  echo
  if [ -n "$analysis_path" ]; then
    analysis_highest="$(jq -r '.summary.highestSeverity // "unknown"' "$analysis_path")"
    analysis_endpoints="$(jq -r '.summary.endpointCount // 0' "$analysis_path")"
    analysis_low="$(jq -r '.summary.findingCounts.LOW // 0' "$analysis_path")"
    analysis_medium="$(jq -r '.summary.findingCounts.MEDIUM // 0' "$analysis_path")"
    analysis_high="$(jq -r '.summary.findingCounts.HIGH // 0' "$analysis_path")"
    analysis_critical="$(jq -r '.summary.findingCounts.CRITICAL // 0' "$analysis_path")"
    echo "- Endpoints analyzed: ${analysis_endpoints}"
    echo "- Highest severity: ${analysis_highest}"
    echo "- Findings: LOW=${analysis_low}, MEDIUM=${analysis_medium}, HIGH=${analysis_high}, CRITICAL=${analysis_critical}"
    echo
    echo "Top analysis findings:"
    jq -r '.endpoints[] | .endpoint as $ep | (.findings[]? | "- [\(.severity)] \(.id) (\($ep)): \(.message)")' "$analysis_path" | head -n 8
  else
    echo "- No analysis artifact provided."
  fi
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
