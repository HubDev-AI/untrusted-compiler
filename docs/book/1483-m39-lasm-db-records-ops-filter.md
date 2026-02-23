# M39: LASM DB records multi-operation filter

## What changed

`/db/records` now supports multi-operation filtering with query param:

- `ops=exec,execTx,queryOne`

Behavior:

- `ops` accepts comma-separated operation names from `exec`, `execTx`, `queryOne`,
- `ops` is applied in addition to existing `op` single-value filter (logical intersection when both are present),
- response filter metadata now includes `ops` as deterministic sorted array,
- invalid `ops` values return deterministic validation error:
  - `400 DB.RECORDS_FILTER_INVALID`
  - `db records ops filter must be comma-separated values from exec, execTx, queryOne`.

## Why

Operator workflows often need combined windows (for example write + query paths) without issuing multiple calls. `ops` supports deterministic multi-op slicing in a single request.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
