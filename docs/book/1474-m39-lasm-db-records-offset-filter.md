# M39: LASM DB records offset filter

## What changed

`/db/records` response materialization now supports `offset` query filter (`>= 0`).

Behavior:

- applies `offset` before list-window materialization,
- works with existing `order` + `limit` filters,
- echoes `offset` in response `filters` metadata.

Invalid `offset` values now return deterministic validation error:

- `400 DB.RECORDS_FILTER_INVALID`
- message: `db records offset filter must be an integer >= 0`

## Why

Without offset support, list-record navigation required hardcoded id/time-range manipulations only. Offset enables straightforward deterministic paging windows over persisted DB operation history.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
