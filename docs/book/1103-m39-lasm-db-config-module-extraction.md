# 1103 M39 Slice: LASM DB Config Module Extraction

This slice starts the adapter-layer extraction plan by moving LASM DB config resolution out of the CLI monolith.

## What changed

1. Added new module: `compiler/sec4-cli/src/lasm_db_config.rs`.
2. Moved DB config/adapter helper logic from `main.rs` into that module:
   - `resolve_lasm_dynamic_store_base`
   - `resolve_lasm_dynamic_db_records_adapter`
   - `resolve_lasm_dynamic_db_tx_max_handles`
   - `load_lasm_db_postgres_dsn_from_file`
   - `resolve_lasm_dynamic_db_postgres_dsn`
   - `lasm_db_records_adapter_label`
3. Kept all call sites in `main.rs` using the same helper behavior and diagnostics.
4. Did not change DB intrinsic semantics, response shapes, or operator flags.

## Why

`compiler/sec4-cli/src/main.rs` accumulated both command dispatch and DB adapter/config mechanics.

Pulling config resolution into a dedicated module creates the first stable seam for adapter packaging/refactoring, while preserving alpha behavior.

## Validation

1. `cargo test -p sec4 --test commands db_max_tx_handles`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_with_cli_flag_when_sqlite_adapter`
