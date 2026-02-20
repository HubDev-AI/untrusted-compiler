# 1096 M39 Slice: LASM DB Record Affected-Rows Persistence

This slice persists `affected_rows` on LASM DB history records so adapter-backed record state and list responses retain write-impact metadata across restarts.

## What changed

1. Extended `LasmDbRecord` with `affected_rows` and propagated it through runtime record creation paths:
   - `db.exec` records persist runtime affected row count
   - `db.execTx` records persist runtime affected row count
   - `db.queryOne` records persist deterministic `affected_rows: 1`
2. Updated records-log JSON conversion:
   - `lasm_db_record_to_json` now emits `affected_rows`
   - `lasm_db_record_from_json` back-compat defaults missing `affected_rows` to `0`
3. Updated sqlite persistence schema and IO:
   - schema includes `affected_rows INTEGER NOT NULL DEFAULT 0`
   - migration path adds the column when loading older sqlite stores
   - sqlite load/insert now read/write `affected_rows`
4. Updated postgres persistence schema and IO:
   - schema includes `affected_rows BIGINT NOT NULL DEFAULT 0`
   - startup schema guard applies `ALTER TABLE ... ADD COLUMN IF NOT EXISTS affected_rows`
   - postgres load/insert now read/write `affected_rows`
5. Strengthened command integration assertions:
   - records-log list/serialized artifacts assert `affected_rows`
   - sqlite row query asserts persisted `affected_rows` column
   - postgres list response asserts `affected_rows` presence

## Why

Before this slice, `affectedRows` existed in write responses but was not consistently persisted in LASM DB record history across all adapters.

Persisting this field closes runtime metadata parity between immediate write responses and durable/listed record history without changing intrinsic semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_maps_sqlite_duplicate_key_exec_to_conflict`
5. `cargo test -p sec4 --test commands run_command_lasm_backend_maps_sqlite_not_null_exec_to_validation`
6. `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_stale_exec_tx_handle_after_restart_when_sqlite_adapter`
