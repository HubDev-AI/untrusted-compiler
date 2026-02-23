# M39: LASM DB records pagination metadata

## What changed

`/db/records` response now includes deterministic pagination metadata:

- `hasMore` (boolean)
- `nextOffset` (integer or `null`)

Metadata is emitted alongside existing record/filter payload and is derived from current list window semantics (`order`, `offset`, `limit`).

## Why

Offset support enables paging, but clients still had to guess whether another page exists and what offset to request next. This change makes paging progression explicit and deterministic from server response metadata.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
