# M39 Browser-to-Server Promotion MVP Execution Plan

## What it is

This chapter converts the post-alpha promotion spec into executable milestone slices (`M39-S1..S4`) with concrete deliverables and test contracts.

It is the implementation plan that starts immediately after `WASM_START_GATE` opens.

## Why it exists

The transformation spec defines the contract, but implementation requires explicit sequencing to avoid rework across compiler semantics, CLI UX, generator behavior, and end-to-end reliability.

M39 defines that sequence as four vertical slices.

## M39 slice plan

### M39-S1: Browser profile capability fence

Deliver:

- Profile-aware semantic checks that block server-only sinks in browser profile.
- Deterministic diagnostics for forbidden capability/effect usage.

Code targets:

- profile model/parse path
- semantic checks for capability/effect constraints
- command fixtures for profile-specific compile failures

Required tests:

1. browser profile rejects `db.*` sink usage.
2. browser profile rejects `secrets.*` usage.
3. browser profile rejects `net.listen` usage.
4. browser profile still allows approved browser-local capabilities.

### M39-S2: Composition contract analyzer

Deliver:

- Promotion-readiness analyzer for domain/repo/adapter split.
- Deterministic diagnostics for contract violations.

Code targets:

- module graph analyzer
- repository interface parity checker
- domain import boundary checker

Required tests:

1. pass fixture with clean domain/repo/adapters split.
2. fail fixture where domain imports server adapter directly.
3. fail fixture where browser/server adapters diverge on required repo methods.

### M39-S3: `sec4 promote` dry-run planner

Deliver:

- CLI command:
  - `sec4 promote --from browser --to server --dry-run`.
- Deterministic plan artifact (`build/promotion-plan.json`) with:
  - binding rewrites,
  - generated files,
  - precondition results.

Code targets:

- CLI command dispatch
- promotion planner engine
- deterministic plan serializer

Required tests:

1. dry-run emits expected plan for valid fixture.
2. dry-run is byte-identical on repeated execution.
3. dry-run fails deterministically when preconditions are violated.

### M39-S4: Promote apply + scaffold generation

Deliver:

- Apply-mode rewrite engine (composition-root-only rewrite guard).
- Deterministic scaffold generation for server target:
  - repo adapter,
  - schema/migration baseline,
  - deploy baseline,
  - promotion report.
- browser-export -> server-import scaffold contract.

Code targets:

- rewrite applier
- scaffold generator
- report/export/import artifact writers

Required tests:

1. apply-mode rewrites only composition binding.
2. domain modules remain unchanged (content hash equality pre/post).
3. generated scaffold files are deterministic.
4. end-to-end fixture:
   - browser prototype builds/runs,
   - promotion succeeds,
   - server build/run succeeds.

## Execution order and gating

M39 execution order is strict:

1. `S1` capability fence
2. `S2` composition analyzer
3. `S3` dry-run planner
4. `S4` apply + scaffold

Gating rules:

- Do not start `S3` until `S1` and `S2` tests are green.
- Do not start `S4` until `S3` deterministic output contract is green.

## Artifacts per slice

- S1: semantic diagnostics + profile fixtures
- S2: analyzer fixtures + parity diagnostics
- S3: promotion plan artifact + planner docs
- S4: generated scaffold tree + promotion report + e2e fixture

## Trade-offs

- Front-loads semantic/architecture checks before generation, reducing downstream rewrite failures.
- Increases early compiler/analyzer scope, but keeps promote/apply deterministic and safer.

## Next

1. Open M39 once `WASM_START_GATE` is satisfied.
2. Execute `S1` and `S2` first as hard prerequisites.
3. Add promotion command scaffolding in `S3`, then complete end-to-end apply path in `S4`.
