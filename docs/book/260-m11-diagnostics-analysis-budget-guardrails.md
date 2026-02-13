# 260 M11 Slice: Diagnostics Analysis Budget Guardrails

This chapter documents bounded-analysis guardrails for diagnostics in `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- diagnostics flow now supports explicit limits:
  - analysis budget (ms),
  - maximum diagnostics per document.
- default limits:
  - `SEC4AUDIT_LSP_ANALYSIS_BUDGET_MS` (default `200`)
  - `SEC4AUDIT_LSP_MAX_DIAGNOSTICS` (default `200`)
- when budget is exceeded, server appends info diagnostic:
  - code `I9001`
  - message indicates results may be incomplete.

## Why it exists

M11 requires bounded execution behavior for editor responsiveness and safety. This slice introduces deterministic guardrails so diagnostic requests cannot silently become unbounded in larger/error-heavy files.

## How it works internally

1. `diagnostics_for_document` now delegates to a limits-aware helper.
2. Diagnostic analysis is timed with `Instant`.
3. Result list is truncated to max-diagnostic limit.
4. If elapsed time meets/exceeds configured budget, append `I9001` informational diagnostic.

## Inputs, outputs, and constraints

- Inputs:
  - document URI + source text,
  - optional budget/cap env vars.
- Outputs:
  - bounded diagnostics list with optional `I9001`.
- Constraints:
  - current guardrail is post-analysis signaling (no mid-analysis cancellation yet).

## Failure modes and diagnostics

- invalid/non-file URI still emits `L9001`.
- budget overflow emits `I9001` to signal partial-risk diagnostics context.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- budget-overflow path appends `I9001` diagnostic via limits-aware helper.

## Tradeoffs and next steps

- Tradeoff:
  - guardrail currently warns after work is done; it does not preempt parser/semantic analysis mid-flight.
- Next:
  - add true request deadline cancellation boundaries around analysis stages,
  - extend guardrails to other LSP flows where needed (completion/rename on large files).
