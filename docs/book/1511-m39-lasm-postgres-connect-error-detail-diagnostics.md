# 1511 M39 Slice: LASM Postgres Connect Error Detail Diagnostics

## What changed

1. Updated LASM Postgres connect error mapping in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - connect failures now include debug-form error details (`{err:?}`) alongside display text for:
     - TLS disabled mode,
     - TLS required mode,
     - auto TLS fallback retry mode.

## Why

Some Postgres startup failures surfaced only as `db error`, which was too opaque for operator troubleshooting in benchmark and runtime startup flows. Including debug error detail exposes concrete root cause data (for example SQLSTATE/auth failures).

## Validation

1. `cargo run -q -p sec4 -- run --path benchmark-suite/services/sec4-lasm --backend lasm --db-adapter postgres --db-postgres-dsn postgres://sec4:wrong@127.0.0.1:5432/sec4_local --oneshot --port 18999`
2. Confirmed stderr now includes detailed Postgres error context (example: `role "sec4" does not exist`, SQLSTATE `E28000`).
