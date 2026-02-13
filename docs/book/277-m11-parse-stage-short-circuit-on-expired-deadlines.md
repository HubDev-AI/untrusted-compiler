# 277 M11 Slice: Parse-Stage Short-Circuit on Expired Deadlines

This chapter documents request-deadline propagation into on-demand parse loading for uncached workspace documents.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- `load_cached_program(...)` now accepts an optional request deadline.
- uncached parse path short-circuits when deadline is already expired.
- all major request paths now thread deadlines into `load_cached_program(...)`:
  - definition/hover/references/completion/prepareRename/rename and workspace declaration scans.

## Why it exists

Before this slice, expired requests could still trigger fresh parse work for uncached files. That violated the intent of strict request budgets and could waste CPU under heavy editor traffic.

## How it works internally

1. Keep fast-path cache return behavior unchanged.
2. Before fallback parsing, check deadline expiration.
3. If expired, return `None` and let caller produce bounded/empty result.
4. Reuse existing request-level deadline wiring across all call sites.

## Inputs, outputs, and constraints

- Inputs:
  - request-scoped `RequestDeadline`.
- Outputs:
  - no on-demand parse work when deadline is expired.
- Constraints:
  - this guards parse entry points, not parser internals mid-execution.

## Failure modes and diagnostics

- expired requests can now return empty/partial editor responses earlier.
- no functional changes for valid-budget requests.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `load_cached_program_skips_parse_when_deadline_is_expired`:
  - verifies uncached parse is skipped for expired deadlines.

## Tradeoffs and next steps

- Tradeoff:
  - cheap pre-parse cancellation is now covered, but parser/analyzer calls are still non-interruptible once started.
- Next:
  - add parser/analyzer-internal interrupt hooks for fully preemptive cancellation semantics.
