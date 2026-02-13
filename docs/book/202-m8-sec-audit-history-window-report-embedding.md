# 202 M8 Slice: sec.audit historyWindow Report Embedding

This chapter documents the M8 step that embeds history-window summaries directly into the canonical audit report JSON.

## What it is

Added optional `historyWindow` to `AuditReport` and wired CLI `sec audit` so runs with `--history-window` include that field in stdout JSON and written report artifacts.

## Why it exists

History-window metrics were previously only auxiliary output (stderr lines and optional sidecar summary files). Embedding them in the report makes the primary audit artifact self-contained for automation and downstream tooling.

## How it works internally

1. Core report schema:
   - `AuditReport` now includes optional `historyWindow: AuditHistoryWindowSummary`.
2. CLI flow:
   - computes history-window summary using core helper
   - attaches it to `report.history_window` before emitting/writing the report
   - continues to emit auxiliary summary lines for human operators.
3. History sampling behavior:
   - includes current run plus up to `window - 1` prior history reports from `--history-dir`.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/audit.rs`
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Output:
  - canonical report JSON now carries `historyWindow` when requested.
- Constraints:
  - requires `--history-window` (and `--history-dir`) to populate.

## Failure modes and diagnostics

- missing required flag combinations still produce usage errors:
  - `--history-window requires --history-dir`
  - `--history-window must be >= 1`
  - `--write-history-summary requires --history-window`

## Example usage

```bash
ailang sec audit \
  --path examples/hello \
  --format json \
  --history-dir .ailang/audit-history \
  --history-window 5
```

The stdout report JSON now includes top-level `historyWindow`.

## Tradeoffs and next steps

- Tradeoff: history computation still lives partly in CLI (history file loading), while summary semantics are core-owned.
- Next:
  - evaluate moving history loading into core and defining stable backward-compat guarantees for `historyWindow`.
