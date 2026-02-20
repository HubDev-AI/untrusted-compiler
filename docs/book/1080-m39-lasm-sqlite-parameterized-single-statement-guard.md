# 1080 M39 Slice: LASM SQLite Parameterized Single-Statement Guard

This slice adds deterministic sqlite runtime safeguards so parameterized DB intrinsic execution cannot run ambiguous multi-statement SQL.

## What changed

1. Added SQL statement-separator scanner that ignores string literals and SQL comments while detecting non-trailing statement separators.
2. Enforced sqlite runtime guard for parameterized execution paths:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
3. Parameterized sqlite SQL containing non-trailing statement separators now fails deterministically with:
   - `sqlite parameterized execution requires a single SQL statement`
4. Added sqlite command integration assertion for queryOne parameterized multi-statement rejection path.

## Why

Parameterized multi-statement execution can produce ambiguous behavior and weaker runtime guarantees.

This keeps sqlite runtime semantics aligned with the stricter Postgres parameterized-path contract while preserving existing non-parameterized multi-statement compatibility.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
