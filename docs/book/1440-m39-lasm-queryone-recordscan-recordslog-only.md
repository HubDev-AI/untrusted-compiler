# M39: LASM QueryOne Record Scan RecordsLog-Only

Date: 2026-02-22  
Milestone: M39 (DB runtime hot-path overhead reduction)

## What Changed

- Updated LASM `db.queryOne` runtime dispatch to avoid unconditional `db_records` reverse-scan on non-records adapters.
- Postgres/SQLite adapter paths now skip history scan and execute adapter-native queryOne flow directly.
- Records-log adapter path keeps reverse-scan match logic (`db`, `template`, `params`) and existing deterministic not-found behavior.

## Why

`db.queryOne` was scanning persisted DB record history on every adapter path, even when Postgres/SQLite adapters return row materialization from live adapter runtime. That scan added lock-held work without affecting Postgres/SQLite outcomes.

## Result

- Lower lock-held work in `db.queryOne` for Postgres/SQLite paths.
- Records-log fallback behavior remains unchanged.
- Runtime response envelopes and persisted record behavior are preserved.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
