# M39: LASM DB records exact id filter

## What changed

`/db/records` now supports exact id filtering via query parameter:

- `id` (`>= 1`)

Behavior:

- applies exact record-id match in list filtering path,
- echoes `id` in response `filters` metadata,
- invalid values return deterministic validation error:
  - `400 DB.RECORDS_FILTER_INVALID`
  - message: `db records id filter must be an integer >= 1`.

## Why

`idFrom`/`idTo` supports range windows, but exact-id lookup is a common operator/debug workflow. Adding `id` provides deterministic single-record access without range math.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
