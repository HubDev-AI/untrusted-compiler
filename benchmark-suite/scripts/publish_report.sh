#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 6 ]; then
  echo "usage: $0 <compare_matrix.json> <out_report.md> [sec_audit.json] [analysis.json] [step_matrix.json] [saturation_summary.md]" >&2
  exit 2
fi

matrix_path="$1"
out_path="$2"
sec_audit_path="${3:-}"
analysis_path="${4:-}"
step_matrix_path="${5:-}"
saturation_summary_path="${6:-}"

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

if [ -n "$step_matrix_path" ] && [ ! -f "$step_matrix_path" ]; then
  echo "step matrix file not found: ${step_matrix_path}" >&2
  exit 2
fi
if [ -n "$saturation_summary_path" ] && [ ! -f "$saturation_summary_path" ]; then
  echo "saturation summary file not found: ${saturation_summary_path}" >&2
  exit 2
fi
if [ -n "$saturation_summary_path" ] && ! grep -Fq -- '- Recommended boost step:' "$saturation_summary_path"; then
  echo "saturation summary missing recommended boost step line: ${saturation_summary_path}" >&2
  exit 2
fi
if [ -n "$saturation_summary_path" ] && ! grep -Fq -- '- Selection mode:' "$saturation_summary_path"; then
  echo "saturation summary missing selection mode line: ${saturation_summary_path}" >&2
  exit 2
fi

extract_summary_value() {
  local prefix="$1"
  local path="$2"
  local line
  line="$(grep -F -- "$prefix" "$path" | head -n 1 || true)"
  line="${line#"$prefix"}"
  printf '%s' "$line"
}

mkdir -p "$(dirname "$out_path")"
now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
endpoint_count="$(jq '.endpoints | length' "$matrix_path")"
endpoint_list="$(jq -r '.endpoints | map(.endpoint) | join(", ")' "$matrix_path")"
impl_count="$(jq '[.endpoints[].compared[].impl] | unique | length' "$matrix_path")"
impl_list="$(jq -r '[.endpoints[].compared[].impl] | unique | join(", ")' "$matrix_path")"
leader_count="$(jq '.endpoints | length' "$matrix_path")"
constant_rate_true_count="$(jq '[.endpoints[].leader | (if has("constantRate") then .constantRate else true end) | select(. == true)] | length' "$matrix_path")"
non_constant_count="$((leader_count - constant_rate_true_count))"
invalid_p99_count="$(jq '[.endpoints[].leader | ((.p99 // "") | tostring | test("[0-9]+(\\.[0-9]+)?") | not)] | map(select(. == true)) | length' "$matrix_path")"
memory_sampled_count="$(jq '[.endpoints[].leader | select((.rssKb // null) != null)] | length' "$matrix_path")"
memory_missing_count="$((leader_count - memory_sampled_count))"
generators="$(jq -r '[.endpoints[].leader | (if has("loadGenerator") then .loadGenerator else "wrk2" end)] | unique | join(", ")' "$matrix_path")"

run_mode="mixed"
if [ "$constant_rate_true_count" -eq "$leader_count" ]; then
  run_mode="constant-rate"
elif [ "$constant_rate_true_count" -eq 0 ]; then
  run_mode="non-constant-rate"
fi

quality_status="PASS"
if [ "$non_constant_count" -gt 0 ] || [ "$invalid_p99_count" -gt 0 ]; then
  quality_status="WARN"
fi
if [ "$memory_missing_count" -gt 0 ]; then
  quality_status="WARN"
fi

{
  echo "# Benchmark Comparative Report (v0.1)"
  echo
  echo "- Generated UTC: ${now_utc}"
  echo "- Matrix source: ${matrix_path}"
  echo "- Implementations in matrix (${impl_count}): ${impl_list}"
  echo "- Endpoints in matrix (${endpoint_count}): ${endpoint_list}"
  echo "- Evidence run mode: ${run_mode}"
  echo "- Evidence quality status: ${quality_status}"
  if [ -n "$sec_audit_path" ]; then
    echo "- Security source: ${sec_audit_path}"
  fi
  if [ -n "$analysis_path" ]; then
    echo "- Analysis source: ${analysis_path}"
  fi
  if [ -n "$step_matrix_path" ]; then
    echo "- Step matrix source: ${step_matrix_path}"
  fi
  if [ -n "$saturation_summary_path" ]; then
    echo "- Saturation summary source: ${saturation_summary_path}"
  fi
  echo

  echo "## Evidence Quality"
  echo
  echo "- Run mode: ${run_mode}"
  echo "- Generators: ${generators}"
  echo "- Memory samples: ${memory_sampled_count}/${leader_count}"
  echo "- Quality status: ${quality_status}"
  if [ "$non_constant_count" -gt 0 ]; then
    echo "- Warning: ${non_constant_count}/${leader_count} endpoint leaders are non-constant-rate (constantRate=false)."
  fi
  if [ "$invalid_p99_count" -gt 0 ]; then
    echo "- Warning: ${invalid_p99_count}/${leader_count} endpoint leaders have missing or non-numeric p99."
  fi
  if [ "$memory_missing_count" -gt 0 ]; then
    echo "- Warning: ${memory_missing_count}/${leader_count} endpoint leaders are missing rssKb memory samples."
  fi
  echo
  echo "| Endpoint | Constant Rate | Generator | p99 | RSS (KB) | Status |"
  echo "|---|---|---|---:|---:|---|"
  jq -r '
    .endpoints[]
    | .endpoint as $ep
    | (.leader // {}) as $l
    | [
        $ep,
        (if ($l | has("constantRate")) then $l.constantRate else true end | tostring),
        (if ($l | has("loadGenerator")) then $l.loadGenerator else "wrk2" end),
        (($l.p99 // "") | tostring),
        (if (($l.rssKb // null) == null) then "n/a" else (($l.rssKb | tostring)) end),
        (
          ((if ($l | has("constantRate")) then $l.constantRate else true end) == true)
          and ((($l.p99 // "") | tostring | test("[0-9]+(\\.[0-9]+)?")))
          and (($l.rssKb // null) != null)
          | if . then "PASS" else "WARN" end
        )
      ]
    | @tsv
  ' "$matrix_path" \
    | while IFS=$'\t' read -r endpoint constant_rate generator p99 rss_kb status; do
        [ -z "$p99" ] && p99="n/a"
        printf "| %s | %s | %s | %s | %s | %s |\n" "$endpoint" "$constant_rate" "$generator" "$p99" "$rss_kb" "$status"
      done
  echo

  echo "## Endpoint Leaders"
  echo
  echo "| Endpoint | Leader | Requests/sec | p99 | RSS (KB) |"
  echo "|---|---|---:|---:|---:|"
  jq -r '.endpoints[] | [.endpoint, (.leader.impl // "n/a"), ((.leader.requestsPerSec // 0)|tostring), (.leader.p99 // ""), (if (.leader.rssKb // null) == null then "n/a" else (.leader.rssKb | tostring) end)] | @tsv' "$matrix_path" \
    | while IFS=$'\t' read -r endpoint leader reqps p99 rss_kb; do
        printf "| %s | %s | %s | %s | %s |\n" "$endpoint" "$leader" "$reqps" "$p99" "$rss_kb"
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
        ((.p99 // "") | tostring | (try capture("(?<n>[0-9]+(\\.[0-9]+)?)").n catch null) // "0" | tonumber);
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

  echo "## Step-Load Signals"
  echo
  if [ -n "$step_matrix_path" ]; then
    step_endpoint_count="$(jq -r '.summary.endpointCount // 0' "$step_matrix_path")"
    step_impl_count="$(jq -r '.summary.implementationCount // 0' "$step_matrix_path")"
    step_knee_count="$(jq -r '.summary.kneeDetectedCount // 0' "$step_matrix_path")"
    echo "- Endpoints analyzed: ${step_endpoint_count}"
    echo "- Implementations compared: ${step_impl_count}"
    echo "- Knee detections: ${step_knee_count}"
    echo
    echo "Per-endpoint leaders (step-load):"
    jq -r '.endpoints[] | "- \(.endpoint): leader=\(.leader.impl), kneeTarget=\(.leader.kneeAtTargetRps), achievedMin=\(((((.leader.achievedRatioMin // 0) * 10000) | round) / 100) )%"' "$step_matrix_path"
  else
    echo "- No step-matrix artifact provided."
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
    echo "- No sec4 audit artifact provided."
  fi
  echo

  echo "## LASM Saturation Boost Tuning"
  echo
  if [ -n "$saturation_summary_path" ]; then
    sat_recommended="$(extract_summary_value '- Recommended boost step: ' "$saturation_summary_path")"
    sat_selection_mode="$(extract_summary_value '- Selection mode: ' "$saturation_summary_path")"
    sat_pass_runs="$(extract_summary_value '- Pass runs: ' "$saturation_summary_path")"
    sat_verify_reqps="$(extract_summary_value '- Requests/sec: ' "$saturation_summary_path")"

    echo "- Summary source: ${saturation_summary_path}"
    echo "- Recommended boost step: ${sat_recommended}"
    echo "- Selection mode: ${sat_selection_mode}"
    echo "- Pass runs: ${sat_pass_runs}"
    if [ -n "$sat_verify_reqps" ]; then
      echo "- Verification requests/sec: ${sat_verify_reqps}"
    fi
  else
    echo "- No saturation-summary artifact provided."
  fi
} > "$out_path"

echo "wrote $out_path"
