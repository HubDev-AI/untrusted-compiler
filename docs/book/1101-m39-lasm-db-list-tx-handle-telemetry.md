# 1101 M39 Slice: LASM DB List Tx-Handle Telemetry

This slice adds live tx-handle state telemetry to LASM DB list responses.

## What changed

1. Extended `DbListRecordsResponse` payload in LASM runtime materialization:
   - `txHandleCount`: current in-memory tx-handle map size
   - `txHandleCapacity`: configured tx-handle capacity (`SEC4_RT_LASM_DB_MAX_TX_HANDLES` / default)
2. Kept existing persisted-record metadata unchanged:
   - `count`
   - `affectedRowsTotal`
   - `adapter`
   - `records`
3. Updated command integration assertions across adapters:
   - records-log list assertions now require tx-handle telemetry fields
   - sqlite list assertions now require tx-handle telemetry fields
   - postgres list assertions now require tx-handle telemetry fields
4. Extended tx-capacity regression assertion:
   - capacity-overflow scenario now asserts deterministic `txHandleCount=1` and `txHandleCapacity=1` in list response.

## Why

Persisted records show historical DB behavior, but operators also need current runtime tx-handle occupancy while debugging transaction pressure and guard behavior.

Adding these fields gives deterministic, low-cost state visibility without changing DB intrinsic semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
