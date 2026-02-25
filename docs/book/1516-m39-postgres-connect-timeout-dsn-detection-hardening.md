# 1516 M39 Slice: Postgres Connect-Timeout DSN Detection Hardening

## What changed

1. Updated `compiler/sec4-cli/src/lasm_db_adapter_state.rs` Postgres DSN timeout rewrite logic:
   - added `has_lasm_postgres_connect_timeout(...)` helper that detects existing `connect_timeout` in a case-insensitive and whitespace-tolerant way.
2. `build_lasm_postgres_connect_dsn(...)` now reuses the helper before injecting timeout parameters.
3. Added unit coverage for:
   - uppercase URL query key (`CONNECT_TIMEOUT=...`),
   - keyword DSN spacing variant (`connect_timeout = ...`),
   - direct helper behavior.

## Why

Previous detection used a case-sensitive direct substring check. DSNs that already carried timeout with case/spacing variants could receive a duplicate injected timeout, creating ambiguous runtime connection settings.

## Validation

1. `cargo test -p sec4 --bin sec4 postgres_connect_timeout_detector_handles_whitespace_and_case -- --nocapture`
2. `cargo test -p sec4 --bin sec4 postgres_connect_dsn_preserves_existing_connect_timeout -- --nocapture`
