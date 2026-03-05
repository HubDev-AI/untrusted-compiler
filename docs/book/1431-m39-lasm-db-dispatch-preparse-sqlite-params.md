# M39: LASM DB Dispatch Pre-Parse SQLite Params

Date: 2026-02-22  
Milestone: M39 (DB runtime contention reduction)

## What Changed

- Updated SQLite runtime entry points to accept pre-parsed parameter slices.
- Updated LASM DB dispatch to pre-parse SQLite params before taking `dynamic_state` lock for:
  - `db.exec`
  - `db.execTx`
  - `db.queryOne`

## Why

SQLite param parsing was still happening inside `dynamic_state` lock after Postgres pre-parse hardening. This kept avoidable JSON/materialization work on the lock-protected path.

## Result

- SQLite DB operations now spend less time under `dynamic_state` lock.
- Postgres and SQLite dispatch paths now follow the same pre-parse lock-scope model.
- DB intrinsic semantics and diagnostics remain unchanged.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 positional_object_params_expand_with_null_fill`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
