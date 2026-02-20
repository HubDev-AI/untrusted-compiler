# 1077 M39 Slice: LASM SQLite Runtime Exec/QueryOne Materialization

This slice upgrades LASM sqlite adapter behavior from metadata-only queryOne fallback to real sqlite query execution paths for DB intrinsics.

## What changed

1. Added sqlite runtime parameter decoding (`sql.q(..., params)` -> rusqlite values) for `db.exec`, `db.execTx`, and `db.queryOne`.
2. Added real sqlite runtime execution helpers:
   - open runtime sqlite store connection with schema bootstrap
   - execute `db.exec` / `db.execTx` inside sqlite transactions
   - execute `db.queryOne` and materialize first-row values to JSON (`null`/number/string/blob base64)
3. Wired LASM intrinsic materialization to execute sqlite runtime paths when `--db-adapter sqlite` is active.
4. Added deterministic sqlite runtime store parent-directory creation before connection open, so first-run intrinsic execution no longer fails on missing directories.
5. Updated sqlite command integration assertions to validate real queryOne row payload materialization instead of legacy metadata fallback row text.

## Why

Alpha DB client behavior requires real adapter execution paths, not metadata-only echoes.  
This slice makes sqlite behavior match the LASM DB-intrinsic contract more closely: runtime execution + typed queryOne row payloads with deterministic envelopes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
