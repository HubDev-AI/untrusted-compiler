# M37 - No-Stub Alpha Readiness Plan

## What this is

This chapter defines the operational meaning of "no-stub alpha" for `Untrusted<T>` and the exact remaining slices to reach it.

## Why this exists

Progress has produced a runnable alpha path, but "runnable" is not the same as "no-stub."  
Without a strict readiness definition, it is easy to over-index on infrastructure/check wiring and under-index on core runtime/compiler behavior.

## Current readiness snapshot (2026-02-15)

- Runnable end-to-end alpha: approximately `80-85%`
- Strict no-stub alpha: approximately `60-70%`
- Current smoke status:
  - `cargo test -p sec4 --test alpha_smoke` is green.

## No-stub alpha criteria

No-stub alpha is ready only when all of the following are true:

1. CLI command surface (`init/check/build/run/test/fmt/lint/audit/replay`) is fully real for supported v0.1 flows.
2. Runtime HTTP server path is fully real across success + deterministic error branches.
3. Security primitives are real for supported scope (URL gates, FS containment, secret flows, ct compare).
4. Outbound NET path covers policy-driven redirect/timeout/body-limit behavior.
5. Policy model/parser/runtime bridge parity is complete for active keys used in execution.
6. Alpha smoke + targeted runtime harness suite pass on `main`.

## Remaining execution slices

### Slice 1: Redirect + policy parity

- Implement runtime redirect handling with deterministic forbidden/limit/invalid branches.
- Complete policy parity for active keys (`json.max_bytes`, `json.max_depth`, `net.public.max_redirects`) and ensure `sec4 run` exports all required runtime envs.

### Slice 2: JSON semantic hardening

- Reduce bridge-only behavior in runtime `json_decode`/`json_encode`.
- Preserve deterministic envelope/error behavior while improving schema-contract semantics.

### Slice 3: Middleware materialization hardening

- Move `fromPolicy` + `with*` middleware paths from toggle-level behavior toward real policy-backed execution.
- Keep deterministic security-first defaults.

### Slice 4: Final no-stub alpha verification

- Run full alpha smoke + targeted runtime harness checks.
- Publish final readiness delta against no-stub criteria.

## Trade-offs

- Prioritizing implementation slices over gate churn increases user-visible progress, but requires tighter discipline in targeted regression testing.
- "No-stub alpha" does not imply production-grade scale; it implies no placeholder behavior in supported v0.1 execution paths.

## Next

1. Execute Slice 1 in parallel lanes (runtime + policy/CLI parity).
2. Integrate and re-evaluate readiness percentage after Slice 1.
