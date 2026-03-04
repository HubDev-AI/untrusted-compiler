# 1084 M39 Slice: `sec4 run --db-postgres-dsn-file` Flag

This slice adds file-based Postgres DSN loading for LASM run mode so operators can avoid inline DSN values while preserving deterministic runtime contracts.

## What changed

1. Added `sec4 run --db-postgres-dsn-file <path>` CLI flag.
2. Added deterministic guardrails:
   - LASM-only usage (`--backend c` rejects the flag)
   - conflict rejection when both `--db-postgres-dsn` and `--db-postgres-dsn-file` are provided
   - non-empty file-content requirement (`must contain a non-empty DSN`)
3. Added runtime file-loading helper with deterministic read/error diagnostics.
4. Wired loaded DSN value through LASM runtime bootstrap and existing Postgres adapter resolution path.
5. Kept env fallback compatibility:
   - if neither explicit DSN flag is provided, runtime still uses `SEC4_DB_ALPHA_DB_POSTGRES_DSN` or `SEC4_RT_LASM_DB_POSTGRES_DSN`.

## Why

File-based DSN wiring improves operational security for CI/deploy setups that mount secrets as files, while preserving explicit run-time operator controls and deterministic validation behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_file_with_c_backend`
3. `cargo test -p sec4 --test commands run_command_rejects_empty_db_postgres_dsn`
4. `cargo test -p sec4 --test commands run_command_rejects_both_db_postgres_dsn_and_file`
5. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
6. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
