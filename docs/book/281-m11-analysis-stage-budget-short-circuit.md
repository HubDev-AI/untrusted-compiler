# 281 M11 Slice: Analysis-Stage Budget Short-Circuit

This chapter documents additional preemptive cancellation in the diagnostics analysis pipeline.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- diagnostics path now short-circuits immediately when analysis budget is zero.
- if parsing consumes the budget, semantic analysis (`analyze_program`) is skipped.
- budget overflow diagnostic emission is centralized through a single helper.

## Why it exists

Request/semantic traversal cancellation was already in place, but diagnostics could still perform parse+analyze work even when budget was already exhausted. This slice enforces stronger budget semantics for diagnostics generation.

## How it works internally

1. At diagnostics entry, return `I9001` immediately when budget is zero.
2. Parse source as usual when budget allows.
3. Before semantic analysis, re-check elapsed budget:
   - if expired, skip analyzer pass and return bounded diagnostics.
4. Append `I9001` when elapsed time exceeds configured budget.

## Inputs, outputs, and constraints

- Inputs:
  - `analysis_budget_ms` and document text.
- Outputs:
  - deterministic, budget-bounded diagnostics generation.
- Constraints:
  - parser/analyzer internals are still not interruptible mid-execution.

## Failure modes and diagnostics

- extremely low budgets may return only the budget marker diagnostic (`I9001`), by design.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `diagnostics_zero_budget_short_circuits_before_parse_errors`:
  - verifies zero-budget mode returns only `I9001` and skips parser diagnostics work.

## Tradeoffs and next steps

- Tradeoff:
  - cancellation is now stronger at stage boundaries but not within parser/analyzer internals.
- Next:
  - add true interrupt hooks inside parser/analyzer execution for full preemptive cancellation.
