# M39: LASM DB Records Limit Window Query

Date: 2026-02-22  
Milestone: M39 (DB runtime operator ergonomics)

## What Changed

- Added `limit` query support on `DbListRecordsResponse` materialization:
  - `GET /db/records?limit=<n>` returns the latest `n` records from runtime history.
  - invalid `limit` values now fail deterministically with `400` and `DB.RECORDS_LIMIT_INVALID`.
- Extended list response telemetry with `recordsTotal` (full retained history count before limit window).
- Kept existing bounded-history telemetry (`recordsCapacity`, `recordsDroppedTotal`) and DB cache/timeout telemetry unchanged.
- Extended command integration coverage to assert deterministic limited-window behavior.

## Why

As DB history grows, operators need a cheap way to inspect recent records without pulling the full retained list each time.

## Result

- `/db/records` now supports lightweight tail-window introspection.
- Clients can distinguish returned window size (`count`) from full retained size (`recordsTotal`).
- Invalid limit input produces deterministic validation diagnostics instead of silent fallback.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
