# M39: Run Flag for DB Record History Capacity

Date: 2026-02-22  
Milestone: M39 (DB runtime operator controls)

## What Changed

- Added new LASM run flag: `--db-records-max <n>`.
- Wired the flag through:
  - direct `sec4 run --backend lasm` execution
  - LASM cluster worker spawn command forwarding
- `--db-records-max` now exports runtime env override `SEC4_RT_LASM_DB_RECORDS_MAX` for the runtime state builder.
- Added deterministic CLI guards:
  - rejected on non-LASM backend
  - rejected when value is `0`

## Why

Operators needed a first-class CLI control for DB history capacity without relying on manual environment-variable setup.

## Result

- DB record history cap can be set explicitly from the run command in both single-instance and cluster flows.
- Invalid usage fails fast with deterministic diagnostics.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/src/lasm_cluster_lifecycle.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 --test commands run_command_rejects_db_records_max_with_c_backend`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_records_max_override`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
