# M39: LASM Postgres Placeholder Index Cache

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Added dynamic-state cache:
  - `db_postgres_placeholder_max_cache: HashMap<String, usize>`
- Added cached resolver in `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`:
  - `max_lasm_postgres_placeholder_index_cached(...)`
- Wired cached placeholder count lookup into:
  - `run_lasm_postgres_exec`
  - `run_lasm_postgres_exec_tx`
  - `run_lasm_postgres_query_one`

## Why

Postgres runtime parameter validation requires computing the highest placeholder index in SQL templates. Previously this scanned SQL text on every call. For repeated templates this added avoidable parsing work.

## Result

- Required-parameter computation is now cached per query template.
- Runtime keeps the same validation behavior and diagnostics; only computation reuse changed.
- Hot Postgres DB paths avoid repeated placeholder rescans.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
