# 1063 M39 Slice: LASM Postgres Client Reuse In Runtime State

This slice removes per-request Postgres reconnect behavior from LASM DB intrinsic execution.

## What changed

1. LASM dynamic DB state now stores a live Postgres client when `--db-adapter postgres` is active.
2. Bootstrap path now:
   - connects once to Postgres,
   - validates schema for `sec4_lasm_db_records`,
   - loads persisted DB metadata records through that same client,
   - keeps the client in runtime state for future DB operations.
3. Runtime DB intrinsic handlers now execute Postgres operations through the in-memory client:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
4. Postgres metadata persistence now uses the same runtime client (no reconnect in persist path).
5. Existing adapters (`records.log`, `sqlite`) remain unchanged.

## Why

The initial Postgres adapter connected on every DB intrinsic call.  
That adds avoidable connection overhead and unstable latency under sustained request load.

Keeping one client per LASM process provides deterministic lower overhead and aligns with the alpha scalability direction.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. Regression checks:
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
