# M39: LASM sqlite pragmas env configuration

## What changed

LASM sqlite adapter bootstrap now supports env-configurable sqlite pragma modes:

- `SEC4_RT_LASM_DB_SQLITE_JOURNAL_MODE` -> `PRAGMA journal_mode`
- `SEC4_RT_LASM_DB_SQLITE_SYNCHRONOUS` -> `PRAGMA synchronous`

Supported values:

- journal mode: `DELETE`, `TRUNCATE`, `PERSIST`, `MEMORY`, `WAL`, `OFF`
- synchronous: `OFF`, `NORMAL`, `FULL`, `EXTRA`

Invalid or empty values now deterministically fall back to the existing defaults:

- journal mode `WAL`
- synchronous `NORMAL`

## Why

The previous runtime path hardcoded `WAL`/`NORMAL` only. This keeps those defaults stable, but lets operators tune sqlite durability/perf behavior per environment without changing language semantics or runtime call surface.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
