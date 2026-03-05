# M39 - Alpha Scope Reset and Multi-File Priority

## Purpose

This chapter records the active scope lock so the team does not repeatedly re-discuss execution priorities in chat.

## Scope Lock (2026-02-17)

The active window is backend no-stub alpha delivery. During this window:

1. Prioritize real runtime/compiler logic over governance-only churn.
2. Keep test loops focused on behavior touched by the current slice.
3. Run broad closure/gate suites near PR completion or in CI, not after each edit.

## Priority Shift

The original `M39-S2` Composition Contract Analyzer is deferred for now. It remains important for browser-to-server promotion, but it is not required to finish backend alpha execution.

The immediate replacement priority is multi-file project support:

1. deterministic project-local module resolution across multiple `.ut` files,
2. deterministic diagnostics for missing modules and ambiguous resolution,
3. deterministic cycle detection for module dependencies.

This priority shift keeps implementation aligned with real authoring needs (single-file-only projects are not sufficient for practical usage).

## Runtime Concurrency Model (Current)

Current generated C runtime behavior is process-scoped request handling with deterministic queue/throttle guards. It is suitable for alpha validation and deterministic behavior checks.

Operational scaling model for now:

1. run multiple runtime processes,
2. place them behind a load balancer/reverse proxy,
3. keep runtime determinism checks unchanged per process.

Planned evolution (post-alpha) can add deeper worker-pool/event-loop internals, but that is outside this scope lock.

Transition reference:

- `docs/book/894-m39-c-runtime-role-and-lasm-async-transition-plan.md`

## Decision Logging Rule

Any future scope pivot must update both:

1. `docs/05-sec4-master-roadmap.md` (canonical agenda/priority state),
2. this book track (chapter-level rationale and trade-offs).

This rule prevents scope decisions from existing only in transient chat threads.
