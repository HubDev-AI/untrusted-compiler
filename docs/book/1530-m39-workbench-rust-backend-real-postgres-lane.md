# 1530 M39 Slice: Workbench Rust Backend Real Postgres Lane

This slice adds the Rust implementation lane to the workbench feature-app matrix.

## What changed

1. Added `benchmark-suite/services/rust-workbench`:
   - `Cargo.toml`
   - `src/main.rs`
   - `smoke.sh`
   - `README.md`
   - `.gitignore`
2. Rust workbench backend behavior:
   - real Postgres operations via `psql` CLI execution,
   - auth-gated mutating routes (`Authorization: Bearer token123`),
   - deterministic success/error envelope contract,
   - transactional route for task + comment creation.
3. Updated workbench matrix status:
   - `benchmark-suite/workbench/matrix.backends.json` marks `rust` as `implemented-alpha`.
4. Workbench smoke orchestration now exercises four implemented lanes:
   - `sec4-lasm`
   - `node`
   - `go`
   - `rust`

## Why

Cross-backend benchmark comparison needs at least three non-sec4 reference lanes before adding the sec4-c runtime lane. Adding Rust extends language/runtime diversity while keeping the same contract and DB behavior.

## Validation

1. `benchmark-suite/services/rust-workbench/smoke.sh`
2. `make -C benchmark-suite workbench-smoke`
