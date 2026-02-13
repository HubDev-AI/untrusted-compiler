# 201 M8 Slice: Core History-Window Summary Model

This chapter documents the M8 refactor that moves history-window summary semantics from CLI-only code into `ailang-core`.

## What it is

Added a core audit model and helper:
- `AuditHistoryWindowSummary`
- `summarize_history_window(reports, window)`

and refactored CLI history-window paths to use this core API.

## Why it exists

History-window behavior was previously duplicated in CLI logic. That made trend semantics harder to reuse and harder to keep consistent across future integrations (CLI, CI tooling, editor/IDE, dashboards). Moving the model into core creates a canonical implementation.

## How it works internally

1. Core (`ailang-core/src/audit.rs`)
   - Defines `AuditHistoryWindowSummary` with serialized field names matching existing summary payload expectations.
   - Implements `summarize_history_window(...)`:
     - oldest/latest/min/max/average risk
     - risk delta
     - oldest/latest policy hash and timestamp anchors
     - severity rollup totals
     - latest-vs-oldest severity deltas
2. Core exports
   - `lib.rs` re-exports the new type and function.
3. CLI (`ailang-cli/src/main.rs`)
   - keeps history file loading in CLI
   - delegates summary computation to core helper
   - uses core struct serialization for stderr and `--write-history-summary`.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/audit.rs`
  - `compiler/ailang-core/src/lib.rs`
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-core/tests/audit_history_window.rs`
- Outputs:
  - canonical core summary model reused by CLI.
- Constraints:
  - report ordering is caller-defined; summaries assume reports are ordered oldest -> latest.

## Failure modes and diagnostics

- Empty history or zero window yields `None` (and CLI enforces strict positive window bounds before calling).
- Regression in summary semantics is caught by new core tests and existing CLI integration tests.

## Example usage

```rust
let summary = summarize_history_window(&reports, 5);
```

`summary` includes risk metrics, severity rollups, and oldest/latest report anchors.

## Tradeoffs and next steps

- Tradeoff: history loading remains CLI-local for now; only summary semantics moved to core.
- Next:
  - evaluate moving history-loading primitives to core and/or embedding window summaries into canonical `AuditReport` schema.
