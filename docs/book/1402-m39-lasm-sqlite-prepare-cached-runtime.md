# M39: LASM SQLite Prepare-Cached Runtime Paths

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Updated LASM SQLite runtime execution/query paths in `compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs`:
  - `run_lasm_sqlite_exec` now uses `tx.prepare_cached(query_template)`
  - `run_lasm_sqlite_query_one` now uses `connection.prepare_cached(normalized_query.as_str())`

## Why

SQLite runtime calls previously prepared statements from SQL text on every execution path. For repeated templates this adds avoidable parse/prepare overhead. `prepare_cached` reuses statement compilation while keeping query semantics unchanged.

## Result

- Repeated SQLite query templates reuse prepared statements within the active connection lifecycle.
- Existing validation/error behavior remains unchanged (`parameter_count`, single-statement checks, row decode paths).
- Runtime hot path does less repeated statement preparation work.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
