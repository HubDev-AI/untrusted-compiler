# M39 Slice: LASM DB Multi-Operation Sequence Dispatch

Date: 2026-02-25  
Milestone: M39 (LASM DB client completion lane)

## What changed

- LASM DB route planning now materializes ordered multi-operation DB plans into indexed internal headers when handlers contain multiple DB intrinsic calls.
- LASM runtime DB dispatch now executes indexed multi-operation sequences in order on the same request path.
- Runtime keeps deterministic behavior on sequence errors:
  - stop on first failed operation,
  - preserve deterministic JSON error envelope classification.
- Single-operation DB routes keep existing behavior unchanged.

## Why

- Multi-operation handlers previously produced route plans with only one operation materialized and were rejected in startup/runtime guard paths.
- Full DB client behavior requires deterministic execution of all DB intrinsics declared in a handler, not last-operation-only semantics.

## Behavioral contract

- For handlers with `N > 1` DB intrinsics:
  - route planning emits `X-Sec4-Internal-Db-Op-Count: N` and indexed DB operation headers.
  - runtime executes operations `0..N-1` in deterministic order.
- Missing/invalid sequence markers return deterministic `DB.OPERATION_INVALID`.
- On full success, response remains materialized by the final operation in the sequence (existing one-operation response shape behavior).

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_executes_multiple_db_intrinsic_ops_in_single_handler -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used -- --exact`
- `cargo test -p sec4 --test commands lasm_smoke_command_materializes_db_list_records_response -- --exact`
