# Alpha Execution Tasks (Implementation-First)

Updated: 2026-03-05  
Branch: `codex/lasm-db-helper-tx-progress`

This is the active execution list for reaching full no-stub alpha readiness.
It is intentionally implementation-focused (runtime/compiler behavior first).

## P0: No-Stub Alpha Closure (Blocking)

- [ ] Eliminate remaining compatibility-style branches on alpha-critical runtime paths.
  - [ ] Keep LASM DB internal operation dispatch orchestration-only.
  - [ ] Keep adapter-specific execution/persistence logic in `lasm_db_client`.
- [ ] Run canonical example flow end-to-end on a clean workspace path:
  - [ ] `sec4 init`
  - [ ] `sec4 check`
  - [ ] `sec4 build --emit lasm`
  - [ ] `sec4 run` + smoke request matrix.
- [ ] Re-run strict no-stub verification bundle on current `dev` baseline and resolve remaining red items.

## P1: Full LASM DB Client Completion (Current Lane)

- [x] Extract runtime adapter/client boundaries into package-style modules (`lasm_db_client`).
- [x] Route `queryOne` records-adapter path through `lasm_db_client` operation entrypoint.
- [x] Route list-records readiness bootstrap through client-layer helper with safe lock scope.
- [ ] Unify `exec`/`execTx` dispatch handling to remove remaining adapter-duplicated branches while preserving deterministic envelopes.
- [ ] Final pass: verify no runtime-dispatch direct adapter internals remain.

## P2: Release Closure (After P0/P1)

- [ ] Refresh roadmap + handoff + book chapter index with final alpha-ready status.
- [ ] Re-run release gate scripts for alpha decision/publish inputs.
- [ ] Prepare final alpha readiness summary with exact pass/fail command evidence.

## Validation Rule (Per Slice)

- [ ] `cargo check -p sec4`
- [ ] At least one focused test for touched behavior path
- [ ] Docs + napkin sync for non-trivial behavior movement

