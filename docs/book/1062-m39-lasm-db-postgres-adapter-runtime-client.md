# 1062 M39 Slice: LASM Real PostgreSQL DB Adapter Client

This slice adds a real PostgreSQL-backed LASM DB adapter path for runtime DB intrinsics.

## What changed

1. Added `postgres` as a first-class LASM DB adapter:
   - CLI: `sec4 run --db-adapter postgres`
   - Env adapter override: `SEC4_RT_LASM_DB_ADAPTER=postgres`
2. Added strict Postgres DSN contract:
   - required env: `SEC4_DB_ALPHA_DB_POSTGRES_DSN` or `SEC4_RT_LASM_DB_POSTGRES_DSN`
   - deterministic startup failure when missing.
3. Extended LASM dynamic DB state to support Postgres metadata storage:
   - adapter label now includes `postgres`,
   - metadata records table: `sec4_lasm_db_records`,
   - startup load/persist parity for record metadata through Postgres.
4. Implemented real Postgres execution for DB intrinsics in LASM run mode:
   - `db.exec` executes query template through Postgres client.
   - `db.execTx` executes query template inside a Postgres transaction.
   - `db.queryOne` executes query template and returns first row data (column/value map serialized in `row`).
5. Kept existing adapters (`records.log`, `sqlite`) intact.

## Why

The previous LASM adapter model persisted deterministic DB metadata locally, but did not provide a real external DB client path.  
This slice unlocks real database execution for alpha LASM flows while preserving existing local adapters for deterministic smoke/CI paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. Regression checks for existing adapters:
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
