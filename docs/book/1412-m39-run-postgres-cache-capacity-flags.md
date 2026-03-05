# M39: Run Flags for Postgres Cache Capacities

Date: 2026-02-22  
Milestone: M39 (DB runtime operator controls)

## What Changed

- Added new LASM run flags:
  - `--db-postgres-statement-cache-max <n>`
  - `--db-postgres-placeholder-cache-max <n>`
- Wired both flags through:
  - direct `sec4 run --backend lasm` execution
  - LASM cluster worker spawn command forwarding
- Extended LASM dynamic state builder to accept explicit cache-capacity inputs from run configuration (instead of requiring env-only overrides).
- Added deterministic CLI guards:
  - rejected on non-LASM backends
  - rejected when value is `0`

## Why

Operators already had env controls for Postgres cache capacities, but runtime tuning from command invocations was inconsistent with other LASM DB controls. These flags make cache-capacity tuning first-class and explicit in run commands.

## Result

- Postgres statement/placeholder cache capacities can be tuned from `sec4 run` in both single-instance and cluster flows.
- Existing env controls remain supported as fallback when flags are not provided.
- Invalid usage fails fast with deterministic diagnostics.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/src/lasm_cluster_lifecycle.rs compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 --test commands run_command_rejects_db_postgres_statement_cache_max_with_c_backend`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_postgres_statement_cache_max_override`
- `cargo test -p sec4 --test commands run_command_rejects_db_postgres_placeholder_cache_max_with_c_backend`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_postgres_placeholder_cache_max_override`
- `cargo test -p sec4 --test commands run_command_rejects_mixed_db_timeout_overrides_without_explicit_adapter`
- `cargo test -p sec4 --test commands run_command_rejects_mixed_db_timeout_overrides_with_explicit_adapter`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
