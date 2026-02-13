# 203 M8 Slice: historyWindow Text Renderer Coverage

This chapter documents test coverage for rendering embedded history-window summaries in text-mode audit output.

## What it is

Added a core test that verifies `render_security_audit_text(...)` includes a `HistoryWindow:` section whenever `AuditReport.history_window` is present.

## Why it exists

After embedding `historyWindow` into the canonical report, text output also gained a history summary line. This test locks that behavior so future refactors do not accidentally drop the section.

## How it works internally

1. Builds a baseline `AuditReport`.
2. Computes a single-report history summary via `summarize_history_window(...)`.
3. Attaches it to `report.history_window`.
4. Renders text and asserts:
   - `HistoryWindow:` line exists
   - `reports=1/1` token appears.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/tests/audit_history_window.rs`
- Output:
  - regression coverage for text-rendered history-window section.
- Constraint:
  - assertion is intentionally format-light (presence checks) to avoid brittle snapshots.

## Failure modes and diagnostics

- If history-window rendering regresses, the test fails with explicit expectation messages for missing section/tokens.

## Example usage

No CLI behavior changes in this slice. It is a pure test hardening increment.

## Tradeoffs and next steps

- Tradeoff: test checks key tokens rather than full line snapshot to keep maintenance low.
- Next:
  - if text output format is stabilized as public contract, consider snapshot-style golden coverage for full renderer lines.
