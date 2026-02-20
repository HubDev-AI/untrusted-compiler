# 1082 M39 Slice: `sec4 run --db-postgres-dsn` Flag

This slice adds explicit CLI DSN wiring for LASM Postgres adapter mode so operators can run Postgres-backed LASM flows without relying only on process environment setup.

## What changed

1. Added `sec4 run --db-postgres-dsn <dsn>` CLI flag.
2. Wired flag through run command dispatch into LASM runtime state initialization.
3. Added cluster forwarding support:
   - fixed-cluster reuse-port mode
   - autoscaled proxy-cluster worker spawn mode
4. Preserved fallback behavior:
   - when `--db-postgres-dsn` is omitted, runtime still reads `SEC4_RT_LASM_DB_POSTGRES_DSN`.
5. Added deterministic validation/guard diagnostics:
   - `--db-postgres-dsn` is LASM-only (`--backend c` fails with status 2)
   - empty `--db-postgres-dsn` values fail deterministically (`must not be empty`)
6. Updated Postgres integration command flow to use `--db-postgres-dsn` directly.

## Why

Explicit run flags improve operator ergonomics and reduce env-only coupling, especially in scripted and clustered runs where forwarding explicit configuration through worker command lines is clearer and easier to audit.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_rejects_empty_db_postgres_dsn`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
5. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
