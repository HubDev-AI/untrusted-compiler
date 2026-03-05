# M39: LASM Postgres Native-TLS Auto Fallback

Date: 2026-02-22  
Milestone: M39 (DB runtime client hardening)

## What Changed

- Added native TLS dependencies for the LASM Postgres runtime path:
  - `native-tls`
  - `postgres-native-tls`
- Added TLS-required error detection for Postgres connection failures.
- Updated Postgres connect bootstrap to:
  - try `NoTls` first,
  - retry automatically with native TLS only when the first error indicates SSL/TLS is required.
- Kept all existing connect-timeout DSN rewriting and session-timeout configuration behavior.
- Added unit coverage for TLS-required error detection patterns.

## Why

Real hosted Postgres deployments often require SSL. The LASM alpha client previously used only `NoTls`, which blocked those deployments unless operators changed infrastructure or used insecure workarounds.

## Result

- Local/non-TLS Postgres flows continue to work unchanged.
- TLS-required Postgres endpoints can connect through automatic native-TLS retry.
- Non-TLS-related connection failures remain deterministic and fail without unnecessary retry loops.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_db_adapter_state.rs`
- `cargo check -p sec4`
- `cargo test -p sec4 postgres_tls_error_detector_`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
