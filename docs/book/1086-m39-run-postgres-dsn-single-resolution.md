# 1086 M39 Slice: Run Postgres DSN Single Resolution

This slice removes duplicate Postgres DSN source reads in LASM run flow by resolving explicit DSN inputs once in `cmd_run` and passing the resolved value downstream.

## What changed

1. Moved explicit DSN-source resolution to `cmd_run`:
   - `--db-postgres-dsn`
   - `--db-postgres-dsn-file`
2. Kept deterministic guard behavior in `cmd_run`:
   - LASM-only flag checks
   - flag-source conflict checks
   - empty DSN / empty DSN-file-content checks
3. Updated `cmd_run_lasm_backend` to consume only the already-resolved explicit DSN value, instead of re-reading DSN files.

## Why

Previously, file-based DSN values were validated in `cmd_run` and then read again in backend startup.  
Single-resolution removes redundant reads and keeps error/exit behavior deterministic in one place.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_empty_db_postgres_dsn`
2. `cargo test -p sec4 --test commands run_command_rejects_both_db_postgres_dsn_and_file`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_with_empty_dsn_file_env`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
