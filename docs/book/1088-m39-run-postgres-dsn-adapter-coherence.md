# 1088 M39 Slice: Run Postgres DSN Adapter Coherence

This slice makes `sec4 run` treat explicit Postgres DSN flags as a coherent adapter contract.

## What changed

1. Updated `cmd_run` adapter selection rules for LASM mode:
   - if `--db-postgres-dsn` or `--db-postgres-dsn-file` is set and `--db-adapter` is omitted, adapter now auto-selects to `postgres`
   - if a DSN flag is set and `--db-adapter` is explicitly non-Postgres, command now fails deterministically with exit `2`
2. Threaded the resolved adapter (`effective_db_adapter`) into backend startup instead of passing raw `db_adapter`.
3. Extended command coverage:
   - Postgres adapter flow now validates DSN flag path without explicitly passing `--db-adapter postgres`
   - added explicit rejection tests for DSN + `sqlite`
   - added explicit rejection tests for DSN-file + `records-log`

## Why

Before this change, DSN flags without an explicit adapter could be ignored by non-Postgres defaults, and DSN + explicit non-Postgres adapter combinations were ambiguous.

The new behavior makes DSN-source intent explicit and deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_with_non_postgres_adapter`
3. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_file_with_non_postgres_adapter`
