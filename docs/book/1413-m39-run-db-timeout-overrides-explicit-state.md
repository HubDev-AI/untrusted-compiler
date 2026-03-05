# M39: Run DB Timeout Overrides Through Explicit Dynamic State

Date: 2026-02-22  
Milestone: M39 (DB runtime operator controls)

## What Changed

- Extended LASM dynamic state builder inputs to accept explicit run-time DB timeout overrides:
  - Postgres statement timeout
  - Postgres lock timeout
  - Postgres connect timeout
  - SQLite busy timeout
- Removed the scoped environment-variable override shim in `cmd_run_lasm_backend` for these run flags.
- Kept env fallback behavior unchanged when flags are not provided.
- Updated deterministic non-Postgres-adapter guard wording to match the broader runtime-override surface (`postgres DSN/runtime overrides ...`).

## Why

Run flags should map directly into runtime state initialization. The previous path depended on temporary env mutation, which made the flag path less direct than other explicit runtime controls.

## Result

- `sec4 run` DB timeout overrides now flow through one explicit state-construction path.
- Runtime behavior remains backward-compatible with existing env defaults.
- Deterministic diagnostics are aligned with the full Postgres runtime-override family.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/tests/commands.rs`
- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_rejects_db_postgres_`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_postgres_lock_timeout_override`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_postgres_connect_timeout_override`
- `cargo test -p sec4 --test commands run_command_rejects_zero_db_sqlite_busy_timeout_override`
- `cargo test -p sec4 --test commands run_command_rejects_mixed_db_timeout_overrides`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
