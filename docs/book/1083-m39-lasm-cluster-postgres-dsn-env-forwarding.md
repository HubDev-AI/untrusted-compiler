# 1083 M39 Slice: LASM Cluster Postgres DSN Env Forwarding

This slice hardens LASM cluster DSN propagation by forwarding Postgres DSNs to worker processes through environment variables instead of command-line arguments.

## What changed

1. Updated LASM cluster worker spawn path to forward explicit Postgres DSN via:
   - `SEC4_RT_LASM_DB_POSTGRES_DSN` environment variable
2. Removed worker command-line `--db-postgres-dsn` forwarding from cluster spawn path.
3. Kept parent entrypoint behavior unchanged:
   - operators can still pass `sec4 run --db-postgres-dsn <dsn>`
   - single-instance LASM runtime still reads explicit DSN directly
4. Preserved DB adapter forwarding and cluster sqlite adapter behavior.

## Why

Forwarding DSNs via worker command-line args exposes sensitive values in process argument listings.

Env propagation reduces that exposure while preserving deterministic runtime behavior and operator ergonomics.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_db_postgres_dsn_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
