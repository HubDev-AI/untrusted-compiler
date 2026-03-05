# M39: LASM DB records window materialization optimization

## What changed

`/db/records` filtering/materialization path now uses index windows instead of cloning the full filtered record set.

Behavior:

- filtering produces a vector of matched record indices,
- order/offset/limit are applied to index windows,
- records are cloned only for selected response window when `includeRecords=true`,
- `includeRecords=false` now avoids record cloning while preserving deterministic summary fields (`count`, totals, pagination metadata).

## Why

Previous implementation cloned every matched record before windowing, even for summary-only responses. This increased hot-path allocation cost as record history grows. Window-index materialization keeps response semantics while reducing cloning overhead.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
