# M39: LASM DB Record History Capacity Guard

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Added bounded in-memory DB record history for LASM dynamic state:
  - new env control `SEC4_RT_LASM_DB_RECORDS_MAX` (default `10000`)
  - oldest records are dropped when capacity is exceeded
- Applied the same capacity bound on startup load so previously persisted large stores do not expand process memory without limit.
- Updated DB operation materialization paths (`exec`, `execTx`, `queryOne`) to append records through shared bounded helper.
- Extended `DbListRecordsResponse` with `recordsCapacity` telemetry.

## Why

`db_records` previously grew without bound in memory. Under long-running or high-throughput workloads this can increase RSS continuously and degrade runtime stability.

## Result

- Runtime memory growth for DB operation history is now deterministic and bounded.
- Recent records are preserved; oldest entries are evicted first.
- Operators can inspect configured bound through `/db/records` response.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 append_db_record_enforces_capacity_by_dropping_oldest`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
