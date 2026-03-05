# Alpha Execution Tasks (Implementation-First)

Updated: 2026-03-05  
Branch: `codex/lasm-db-helper-tx-progress`

This is the active execution list for reaching full no-stub alpha readiness.
It is intentionally implementation-focused (runtime/compiler behavior first).

## P0: No-Stub Alpha Closure (Blocking)

- [x] Eliminate remaining compatibility-style branches on alpha-critical runtime paths.
  - [x] Keep LASM DB internal operation dispatch orchestration-only.
  - [x] Keep adapter-specific execution/persistence logic in `lasm_db_client`.
- [x] Run canonical example flow end-to-end on a clean workspace path:
  - [x] `sec4 run` + smoke request matrix (`examples/lasm-alpha-full/scripts/run-smoke.sh`, records adapter).
  - [x] `sec4 init`
  - [x] `sec4 check`
  - [x] `sec4 build --emit lasm`
- [x] Re-run strict no-stub verification bundle on current `dev` baseline and resolve remaining red items.

## P1: Full LASM DB Client Completion (Current Lane)

- [x] Extract runtime adapter/client boundaries into package-style modules (`lasm_db_client`).
- [x] Route `queryOne` records-adapter path through `lasm_db_client` operation entrypoint.
- [x] Route list-records readiness bootstrap through client-layer helper with safe lock scope.
- [x] Unify `exec`/`execTx` dispatch handling to remove remaining adapter-duplicated branches while preserving deterministic envelopes.
- [x] Final pass: verify no runtime-dispatch direct adapter internals remain.

## P2: Release Closure (After P0/P1)

- [x] Refresh roadmap + handoff + book chapter index with final alpha-ready status.
- [x] Re-run release gate scripts for alpha decision/publish inputs.
- [x] Prepare final alpha readiness summary with exact pass/fail command evidence.

## Validation Rule (Per Slice)

- [x] `cargo check -p sec4`
- [x] At least one focused test for touched behavior path
- [x] Docs + napkin sync for non-trivial behavior movement

## Latest Verification Evidence (2026-03-05)

- `cargo check -p sec4` -> pass
- `cargo test -p sec4 lasm_db_runtime_dispatch::tests::list_records_marker_materializes_records_payload -- --exact` -> pass
- `cargo test -p sec4 --test commands promote_apply_rewrites_composition_root_and_generates_scaffold -- --exact` -> pass
- `cargo test -p sec4 --test commands promote_dry_run_reports_blocking_preconditions_for_invalid_project -- --exact` -> pass
- `scripts/release-alpha-gate.sh` -> pass (`ok: alpha release gate passed`)
