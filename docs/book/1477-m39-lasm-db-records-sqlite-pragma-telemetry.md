# M39: LASM DB records sqlite pragma telemetry

## What changed

`/db/records` telemetry now exposes effective sqlite pragma settings in `dbTimeoutsMs`:

- `sqliteJournalMode`
- `sqliteSynchronous`

Values are sourced from runtime sqlite pragma resolution (`SEC4_RT_LASM_DB_SQLITE_JOURNAL_MODE` / `SEC4_RT_LASM_DB_SQLITE_SYNCHRONOUS`, with deterministic defaults/fallback).

## Why

Runtime sqlite behavior can now be tuned by environment. Exposing effective pragma values in DB records telemetry makes operator inspection straightforward without shell/env introspection.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
