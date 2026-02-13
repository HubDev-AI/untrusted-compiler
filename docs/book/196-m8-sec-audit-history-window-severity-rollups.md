# 196 M8 Slice: sec.audit History-Window Severity Rollups

This chapter documents the follow-up M8 enhancement that adds severity rollups to the history-window trend summary.

## What it is

Extended `--history-window` summary output with:
- `averageRiskScore`
- `highestSeveritySeen`
- `severityRollup` totals across the sampled window
- `severityLatestDelta` (latest minus oldest severity counts)

## Why it exists

Risk-score drift alone is not enough to explain posture movement. Severity rollups provide a clearer view of whether the window trend is driven by low-noise findings or high/critical posture regressions.

## How it works internally

1. While loading recent history reports, the CLI now aggregates severity counts (`LOW`, `MEDIUM`, `HIGH`, `CRITICAL`) across the sampled window.
2. It computes latest-vs-oldest deltas per severity level.
3. It computes highest severity seen in the window and average risk score.
4. In JSON output mode, these values are emitted as structured payload in the stderr summary line:
   - `history window summary: {...}`

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Output:
  - richer history-window auxiliary summary in `sec audit`.
- Constraints:
  - values remain auxiliary output; core audit report schema is unchanged.

## Failure modes and diagnostics

- If rollup fields regress, CLI integration tests fail because JSON-mode stderr summary no longer contains `severityRollup` / `severityLatestDelta`.

## Example usage

```bash
ailang sec audit \
  --path examples/hello \
  --format json \
  --history-dir .ailang/audit-history \
  --history-window 5
```

Expected stderr now includes rollup keys:
- `severityRollup`
- `severityLatestDelta`

## Tradeoffs and next steps

- Tradeoff: summary remains CLI-emitted metadata, so downstream automation must parse stderr if it needs these window rollups.
- Next:
  - move multi-run aggregates into a stable report artifact once output schema expansion is approved.
