# M39: Run Flag for Postgres TLS Mode

Date: 2026-02-22  
Milestone: M39 (DB runtime operator controls)

## What Changed

- Added new LASM run flag:
  - `--db-postgres-tls-mode <auto|disable|require>`
- Wired TLS mode through:
  - direct `sec4 run --backend lasm` execution
  - LASM cluster worker spawn command forwarding
  - explicit dynamic-state initialization path
  - Postgres reconnect path
- Added TLS mode resolution fallback from env:
  - `SEC4_RT_LASM_DB_POSTGRES_TLS_MODE` (`auto` default)
- Added deterministic mode labels in `/db/records` telemetry:
  - `dbTimeoutsMs.postgresTlsMode`
- Added deterministic CLI guards:
  - rejected on non-LASM backends
  - rejected with non-Postgres adapters when explicitly set

## Why

TLS auto fallback is useful, but operators also need explicit control for strict environments:
- force plaintext for local/trusted internal setups
- force TLS in hardened environments
- keep deterministic behavior across cluster workers

## Result

- Postgres TLS behavior is now explicitly configurable per run.
- The selected mode is observable in runtime DB telemetry.
- Runtime connect/reconnect paths stay aligned under one mode contract.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/src/lasm_db_adapter_state.rs compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_common.rs compiler/sec4-cli/src/lasm_db_cli.rs compiler/sec4-cli/src/lasm_cluster_lifecycle.rs compiler/sec4-cli/tests/commands.rs`
- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_rejects_db_postgres_`
- `cargo test -p sec4 postgres_tls_mode_`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
