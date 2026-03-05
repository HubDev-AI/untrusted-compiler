# M39: LASM sqlite pragmas WAL/NORMAL

## What changed

LASM sqlite adapter connection bootstrap now configures sqlite pragmas for runtime behavior:

- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- `PRAGMA synchronous = NORMAL`

These pragmas are now applied in the shared sqlite connect path used by:

- dynamic-state bootstrap sqlite connection,
- sqlite reconnect paths in runtime append/query flows.

## Why

The sqlite adapter is now part of the real LASM DB path. Enabling WAL journaling with NORMAL sync improves practical embedded DB runtime behavior under mixed read/write request traffic while keeping deterministic DB intrinsic semantics.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
