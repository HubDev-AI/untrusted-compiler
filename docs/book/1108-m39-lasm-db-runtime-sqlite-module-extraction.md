# 1108 M39 Slice: LASM DB Runtime SQLite Module Extraction

This slice complements postgres runtime extraction by moving sqlite runtime DB execution helpers out of `main.rs`.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs`.
2. Moved sqlite runtime helpers from `main.rs`:
   - sqlite param parsing/coercion helpers
   - sqlite runtime connection bootstrap helper
   - sqlite row-value JSON conversion helper
   - sqlite parameter-arity validator
   - `run_lasm_sqlite_exec`
   - `run_lasm_sqlite_exec_tx`
   - `run_lasm_sqlite_query_one`
3. `main.rs` now imports sqlite runtime helpers from module and keeps shared SQL statement-separator guard in place.
4. Kept runtime behavior unchanged for sqlite exec/queryOne paths.

## Why

SQLite runtime execution is adapter-specific logic and should live with adapter runtime helpers rather than command/orchestration code.

This extraction significantly reduces DB runtime noise in `main.rs` and aligns sqlite/postgres module boundaries.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
