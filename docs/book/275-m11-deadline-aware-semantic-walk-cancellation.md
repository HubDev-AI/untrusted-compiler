# 275 M11 Slice: Deadline-Aware Semantic Walk Cancellation

This chapter documents extending LSP request deadline enforcement into semantic traversal routines.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- request-deadline checks now propagate into:
  - function symbol collection,
  - identifier-at-position lookup traversal,
  - identifier reference-hit collection traversal.
- definition/hover/references/rename/completion now pass request deadlines into these semantic walks.
- expired requests now short-circuit semantic scans deterministically instead of traversing full ASTs.

## Why it exists

Loop-level deadline checks already limited workspace scan loops, but deep AST walks could still do unnecessary work on large files. Propagating deadlines into semantic traversal keeps LSP behavior responsive and aligned with budget guarantees.

## How it works internally

1. Add optional deadline parameter to semantic walkers.
2. Check deadline at walker entry and inside recursive/iterative branches.
3. Stop traversal immediately on expiration and return partial/empty results safely.
4. Thread deadlines from LSP request handlers into the updated traversal APIs.

## Inputs, outputs, and constraints

- Inputs:
  - `RequestDeadline` derived from `SEC4AUDIT_LSP_REQUEST_BUDGET_MS`.
- Outputs:
  - deterministic best-effort results within request budget.
- Constraints:
  - cancellation is cooperative (checks between traversal steps).

## Failure modes and diagnostics

- expired requests may return partial/empty semantic outputs (for example fewer completion symbols or no identifier hit) rather than hanging.
- no crash path introduced; behavior degrades to bounded results.

## Tests added/updated

`compiler/sec4-lsp` now includes:
- `deadline_aware_semantic_walks_can_short_circuit`:
  - validates immediate short-circuit for symbol collection, identifier lookup, and hit collection with an expired deadline.

## Tradeoffs and next steps

- Tradeoff:
  - cooperative checks improve responsiveness but do not interrupt parser/analyzer internals mid-call.
- Next:
  - add parser/analyzer-stage interrupt support for fully preemptive cancellation across the entire request pipeline.
