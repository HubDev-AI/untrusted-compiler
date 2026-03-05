# 1518 M39 Slice: Postgres Connect Diagnostic DSN Redaction

## What changed

1. Updated Postgres connect error formatting in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - added DSN-password redaction helper for URL DSNs,
   - added connect-error message redaction pass that replaces embedded DSN instances with redacted form.
2. Applied redaction on all Postgres connect diagnostic branches:
   - `disable`,
   - `require`,
   - `auto` (NoTLS + TLS retry detail path).
3. Added focused unit coverage for DSN redaction and message replacement helpers.

## Why

Postgres connect diagnostics now intentionally include richer debug context (`{err:?}`). When backend libraries echo DSN strings, raw credentials may appear in error text. This hardening keeps diagnostics actionable while reducing accidental credential disclosure.

## Validation

1. `cargo test -p sec4 --bin sec4 redact_postgres_dsn_password_masks_url_credentials -- --nocapture`
2. `cargo test -p sec4 --bin sec4 redact_postgres_connect_error_message_replaces_embedded_dsn -- --nocapture`
