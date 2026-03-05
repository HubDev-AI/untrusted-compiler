# 1501 M39 Slice: Run DB CLI Validation + Resolution Module Extraction

## What changed

1. Extended `compiler/sec4-cli/src/lasm_db_cli.rs` with DB-run option resolution helper:
   - `validate_and_resolve_run_db_cli_options(...)`
   - `ResolvedRunDbCliOptions`
2. Moved DB-specific `sec4 run` guard/normalization logic out of `main.rs` into this helper:
   - LASM-only DB flag guards (`--db-*`)
   - DB numeric lower-bound guards and platform-size checks
   - sqlite mode normalization (`--db-sqlite-journal-mode`, `--db-sqlite-synchronous`)
   - Postgres DSN source validation (`--db-postgres-dsn` / `--db-postgres-dsn-file`)
   - sqlite/postgres runtime override coherence and effective adapter selection
3. Updated `cmd_run` to delegate DB option validation/resolution to `lasm_db_cli` and keep the same deterministic diagnostics/exit contract.

## Why

`cmd_run` still contained a large DB-only validation block despite prior DB runtime extraction work. Moving this block into `lasm_db_cli` keeps command orchestration thinner and strengthens the DB package/module boundary without changing runtime semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_db_`
2. `cargo test -p sec4 --test commands run_command_rejects_zero_db_`
3. `cargo test -p sec4 --test commands run_command_rejects_mixed_db_timeout_overrides`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
5. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
