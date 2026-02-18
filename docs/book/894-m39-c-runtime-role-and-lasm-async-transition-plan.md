# M39 - C Runtime Role and LASM Async Transition Plan

## Why This Chapter Exists

Backend direction needed a hard decision:

1. C runtime is useful for alpha correctness,
2. but one-thread/process-scoped behavior is not the long-term server architecture.

This chapter records the transition plan so the team does not keep re-arguing the same scope question.

## Decision (2026-02-17)

1. Keep C runtime as the no-stub alpha reference engine.
2. Do not over-invest in C runtime architecture beyond alpha-critical behavior.
3. Start LASM async backend bootstrap on February 18, 2026 in parallel.

## Time Split Through March 17, 2026

1. `70%` delivery effort: alpha closure + multi-file modules.
2. `30%` delivery effort: LASM async backend bootstrap.

This split keeps alpha momentum while preventing backend architecture lock-in.

## Scope Boundaries

### C runtime (reference lane)

Allowed:

1. correctness fixes,
2. deterministic error/response behavior,
3. alpha contractual parity checks.

Not prioritized:

1. deep async re-architecture in C,
2. full server-scale tuning investments.

### LASM runtime (future server lane)

Bootstrap objectives:

1. backend target skeleton and compile path wiring,
2. async reactor/scheduler baseline,
3. minimal concurrent HTTP request/response path with deterministic envelopes.

## Browser/WASM Clarification

Browser target is a separate runtime profile (WASM + browser host ABI). It is not the socket-based C server runtime moved unchanged into the browser.

That separation is intentional:

1. LASM async runtime solves server concurrency evolution.
2. WASM runtime profile solves browser execution constraints.

## Checkpoint Rule (March 17, 2026)

1. If alpha no-stub criteria are green, continue release hardening.
2. If LASM bootstrap demonstrates viable concurrent request handling, start staged migration planning from C reference runtime to LASM server runtime.
3. If LASM is not yet viable, keep C as alpha reference and continue LASM in bounded slices.

## Practical Outcome

This plan avoids two failure modes:

1. over-building C runtime architecture that will be replaced,
2. abandoning alpha delivery before a credible LASM baseline exists.
