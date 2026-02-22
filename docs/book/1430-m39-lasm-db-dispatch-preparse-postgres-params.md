# M39: LASM DB Dispatch Pre-Parse Postgres Params

Date: 2026-02-22  
Milestone: M39 (DB runtime contention reduction)

## What Changed

- Updated LASM DB dispatch paths for Postgres adapter:
  - `db.exec`
  - `db.execTx`
  - `db.queryOne`
- Postgres SQL params are now parsed before entering `dynamic_state` lock.
- Lock-protected runtime execution now consumes precomputed parameter vectors.

## Why

Postgres param parsing is pure JSON/materialization work and does not require mutable runtime state. Keeping that work inside `dynamic_state` lock increased contention and extended lock hold time on DB-heavy request paths.

## Result

- Less time spent inside `dynamic_state` lock on Postgres DB operations.
- Existing DB intrinsic contracts and diagnostics remain unchanged.
- Runtime behavior for sqlite/records adapters is unaffected.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
