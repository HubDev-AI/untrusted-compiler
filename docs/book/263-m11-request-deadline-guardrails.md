# 263 M11 Slice: Request-Deadline Guardrails

This chapter documents preemptive request-deadline checks for multi-document LSP operations.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- Added request deadline model:
  - `RequestDeadline { started_at, budget_ms }`
  - `request_budget_ms()` config via `SEC4AUDIT_LSP_REQUEST_BUDGET_MS` (fallback to analysis budget).
- Applied deadline checks in multi-document loops:
  - declaration lookup across workspace-open documents,
  - references aggregation,
  - rename edit aggregation.

## Why it exists

M11 requires bounded behavior to avoid perceived hangs in editor flows. References/rename can scan multiple open documents; deadline checks provide deterministic early cutoff when request budget is exceeded.

## How it works internally

1. Request handlers construct deadline from configured budget.
2. Before each document-level parse/scan step, check `deadline.is_expired()`.
3. If expired:
  - stop scanning further documents,
  - return best-effort partial result collected so far.

## Inputs, outputs, and constraints

- Inputs:
  - references/rename requests with open-document workspace state.
- Outputs:
  - deterministic best-effort responses under budget.
- Constraints:
  - deadline guardrails currently apply to document-loop stages; parser/semantic internals are not preempted mid-call.

## Failure modes and diagnostics

- low request budgets can intentionally truncate cross-document aggregation.
- behavior remains deterministic because document iteration is URI-sorted.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- zero-budget deadline semantics (`RequestDeadline::new(0)` is immediately expired).

## Tradeoffs and next steps

- Tradeoff:
  - guardrails are cooperative and stage-level; no hard interrupt inside parse/analyze calls.
- Next:
  - expose partial-result markers in LSP responses/diagnostics metadata when deadline truncation occurs,
  - add deeper cancellation points in heavier analysis paths as compiler APIs evolve.
