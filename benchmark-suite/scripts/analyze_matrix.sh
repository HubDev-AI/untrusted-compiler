#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <compare_matrix.json> <out_analysis.json>" >&2
  exit 2
fi

matrix_path="$1"
out_path="$2"

if [ ! -f "$matrix_path" ]; then
  echo "compare matrix file not found: ${matrix_path}" >&2
  exit 2
fi

if [ "$(jq '.endpoints | length' "$matrix_path")" -eq 0 ]; then
  echo "compare matrix has no endpoints: ${matrix_path}" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"

jq '
  def p99num($v):
    (($v | tostring | capture("(?<n>[0-9]+(\\.[0-9]+)?)")?.n) // "0" | tonumber);

  def sev_rank($s):
    if $s == "CRITICAL" then 4
    elif $s == "HIGH" then 3
    elif $s == "MEDIUM" then 2
    else 1 end;

  . as $matrix
  | {
      version: "0.1",
      endpoints: (
        ($matrix.endpoints // [])
        | map(
            . as $ep
            | ($ep.compared // []) as $rows
            | ($ep.leader // ($rows[0] // null)) as $leader
            | ($rows | map(p99num(.p99))) as $p99_vals
            | ($p99_vals | if length > 0 then min else 0 end) as $p99_min
            | ($p99_vals | if length > 0 then max else 0 end) as $p99_max
            | (if $p99_min > 0 then ($p99_max / $p99_min) else null end) as $p99_spread
            | ($leader.targetRps // 0) as $target_rps
            | ($leader.requestsPerSec // 0) as $leader_rps
            | (if $target_rps > 0 then (($leader_rps / $target_rps) * 100) else null end) as $coverage
            | {
                endpoint: $ep.endpoint,
                leader: $leader,
                metrics: {
                  comparedCount: ($rows | length),
                  p99MinMs: $p99_min,
                  p99MaxMs: $p99_max,
                  p99SpreadX: $p99_spread,
                  leaderTargetCoveragePct: $coverage
                },
                findings: [
                  (
                    if $p99_spread == null then empty
                    elif $p99_spread >= 2.5 then {
                      id: "P99_SPREAD_HIGH",
                      severity: "HIGH",
                      message: "p99 spread is >= 2.5x across implementations",
                      evidence: {p99SpreadX: $p99_spread}
                    }
                    elif $p99_spread >= 1.5 then {
                      id: "P99_SPREAD_MEDIUM",
                      severity: "MEDIUM",
                      message: "p99 spread is >= 1.5x across implementations",
                      evidence: {p99SpreadX: $p99_spread}
                    }
                    else empty end
                  ),
                  (
                    if $coverage == null then empty
                    elif $coverage < 90 then {
                      id: "LEADER_TARGET_COVERAGE_LOW",
                      severity: "HIGH",
                      message: "leader achieved < 90% of target RPS",
                      evidence: {targetRps: $target_rps, requestsPerSec: $leader_rps, coveragePct: $coverage}
                    }
                    elif $coverage < 95 then {
                      id: "LEADER_TARGET_COVERAGE_WARN",
                      severity: "MEDIUM",
                      message: "leader achieved < 95% of target RPS",
                      evidence: {targetRps: $target_rps, requestsPerSec: $leader_rps, coveragePct: $coverage}
                    }
                    else empty end
                  ),
                  (
                    if ($rows | any((.requestsPerSec // 0) <= 0)) then {
                      id: "ZERO_THROUGHPUT_IMPLEMENTATION",
                      severity: "HIGH",
                      message: "one or more implementations reported zero throughput",
                      evidence: {
                        implementations: (
                          $rows
                          | map(select((.requestsPerSec // 0) <= 0) | .impl)
                        )
                      }
                    }
                    else empty end
                  )
                ]
              }
          )
      )
    }
  | . as $analysis
  | ($analysis.endpoints | map(.findings[]) ) as $all_findings
  | .summary = {
      endpointCount: ($analysis.endpoints | length),
      findingCounts: {
        LOW: ($all_findings | map(select(.severity == "LOW")) | length),
        MEDIUM: ($all_findings | map(select(.severity == "MEDIUM")) | length),
        HIGH: ($all_findings | map(select(.severity == "HIGH")) | length),
        CRITICAL: ($all_findings | map(select(.severity == "CRITICAL")) | length)
      },
      highestSeverity: (
        if ($all_findings | length) == 0 then "LOW"
        else ($all_findings | max_by(sev_rank(.severity)) | .severity)
        end
      )
    }
' "$matrix_path" > "$out_path"

echo "wrote $out_path"
