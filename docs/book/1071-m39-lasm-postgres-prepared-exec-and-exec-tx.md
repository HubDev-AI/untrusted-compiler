# 1071 M39 Slice: LASM Postgres Prepared Exec and ExecTx

This slice upgrades LASM Postgres write-side intrinsic execution (`db.exec`, `db.execTx`) to use real prepared parameter binding.

## What changed

1. Added typed Postgres parameter parsing model for `sql.q(..., params)` values used in LASM runtime:
   - strings
   - numbers
   - booleans
   - null
2. Added parameter-reference bridge (`ToSql`) for runtime Postgres client execution.
3. `db.exec` now uses prepared execution (`client.execute`) when SQL placeholders/params are present; no-param path keeps `batch_execute` behavior.
4. `db.execTx` now uses prepared execution (`transaction.execute`) when SQL placeholders/params are present; no-param path keeps `batch_execute` behavior.
5. Added deterministic guidance for unsupported parameterized multi-statement SQL (`postgres parameterized execution requires a single SQL statement`).
6. Updated Postgres command integration fixture to exercise placeholder-based `db.exec` request path.

## Why

Prepared execution is required for real DB-client behavior and avoids string-substitution semantics on write paths.

This keeps LASM Postgres adapter closer to production DB client expectations while preserving no-param compatibility for existing SQL flows.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
