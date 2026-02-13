# 197 M8 Slice: sec4 audit History-Summary Export

This chapter documents the M8 follow-up that persists history-window metrics to a JSON artifact.

## What it is

Added `--write-history-summary <path>` to `sec4 audit`.

When combined with `--history-dir` and `--history-window`, the command now writes a JSON file containing the computed multi-run window summary (risk metrics + severity rollups/deltas).

## Why it exists

History-window output on stderr is useful for humans, but CI/reporting pipelines need a stable file artifact they can ingest directly. This slice enables deterministic export without changing the main stdout audit report schema.

## How it works internally

1. CLI contract:
   - `--write-history-summary` requires `--history-window`.
   - `--history-window` requires `--history-dir`.
2. The same computed `HistoryWindowSummary` used for stderr output is serialized to JSON and written to the requested path.
3. Parent directories are created automatically.
4. On success, CLI emits an auxiliary line with the output path:
   - `history summary: <path>`

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-cli/src/main.rs`
  - `compiler/sec4-cli/tests/json_output.rs`
- Output:
  - JSON summary artifact containing:
    - window/reports
    - oldest/latest/min/max/average risk
    - risk delta
    - highest severity seen
    - `severityRollup`
    - `severityLatestDelta`
- Constraints:
  - export is only available when a history-window computation is active.

## Failure modes and diagnostics

- Missing required companion flags:
  - `--write-history-summary requires --history-window`
  - `--history-window requires --history-dir`
- File I/O and serialization errors are surfaced with explicit file-path diagnostics and non-zero exit code.

## Example usage

```bash
sec4 audit \
  --path examples/hello \
  --format json \
  --history-dir .sec4/audit-history \
  --history-window 10 \
  --write-history-summary artifacts/audit/history-window.json
```

## Tradeoffs and next steps

- Tradeoff: summary export is CLI-layer metadata and currently separate from the core `AuditReport` schema.
- Next:
  - decide whether to embed these multi-run metrics in the core report model for first-class downstream consumption.
