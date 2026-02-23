# M39: LASM DB records limit max guard

## What changed

`/db/records` `limit` filter is now bounded to `[1..1000]`.

Behavior:

- accepted values: integer `>= 1` and `<= 1000`,
- invalid values return deterministic:
  - `400 DB.RECORDS_LIMIT_INVALID`
  - `db records limit must be an integer between 1 and 1000`,
- response filter metadata now echoes `limit`.

## Why

Unbounded list limits can produce oversized payloads and unpredictable runtime cost under high operation-history volumes. The max guard keeps DB records response sizing deterministic for alpha runtime operations.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
