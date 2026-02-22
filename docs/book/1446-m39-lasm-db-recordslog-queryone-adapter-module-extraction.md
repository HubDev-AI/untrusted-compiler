# M39: LASM DB Records-Log QueryOne Adapter Module Extraction

Date: 2026-02-22  
Milestone: M39 (DB adapter-layer extraction)

## What Changed

- Added new runtime module:
  - `compiler/sec4-cli/src/lasm_db_runtime_records_log.rs`.
- Moved records-log adapter-specific `queryOne` logic into that module:
  - latest signature match lookup (`find_lasm_records_log_latest_match`),
  - row metadata materialization (`build_lasm_records_log_query_one_row_object`).
- Updated runtime dispatch wiring:
  - `lasm_db_runtime_dispatch.rs` now delegates records-log-specific lookup/materialization through helper calls.

## Why

Records-log adapter logic was embedded directly in the shared DB dispatch path. This made adapter boundaries noisier and slowed planned extraction of adapter layers into clearer module/package seams.

## Result

- Dispatch path is thinner and less adapter-specific.
- Records-log `queryOne` behavior is preserved.
- Adapter extraction track is incrementally advanced without changing intrinsic semantics.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
