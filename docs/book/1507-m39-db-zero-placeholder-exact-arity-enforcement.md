# 1507 M39 Slice: DB Zero-Placeholder Exact-Arity Enforcement

## What changed

1. Removed zero-placeholder compatibility bypass from runtime arity checks:
   - `validate_lasm_sqlite_parameter_arity(...)` no longer accepts extra params when SQL has no placeholders,
   - `validate_lasm_postgres_parameter_arity(...)` no longer accepts extra params when SQL has no placeholders.
2. Updated runtime unit tests to assert deterministic strict arity errors for zero-placeholder SQL with extra params.
3. Updated LASM DB command integration fixtures to send explicit empty params payloads (`[]`) on zero-placeholder SQL success paths.

## Why

This removes a remaining compatibility-only branch from alpha-critical DB runtime behavior and keeps arity contracts strict and uniform across placeholder and zero-placeholder SQL templates.

## Validation

1. `cargo test -p sec4 postgres_parameter_arity_rejects_extra_params_when_sql_has_no_placeholders`
2. `cargo test -p sec4 positional_arity_validation_rejects_extra_params_when_sql_has_no_placeholders`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
5. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_with_cli_flag_when_sqlite_adapter`
