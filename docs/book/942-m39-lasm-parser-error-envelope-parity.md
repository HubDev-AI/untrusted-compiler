# M39: LASM Parser Error Envelope Parity

## Why

LASM parser failures were still emitted as plain-text responses, while other LASM runtime failures already used structured JSON error envelopes with stable fields.

That split made error handling less consistent for clients and harder to reason about contract parity.

## What Changed

In `compiler/sec4-cli/src/main.rs` (`process_lasm_connection_with_runtime`):

1. Request-parse failures (except the empty-request early return) now emit JSON envelopes using:
   - `set_lasm_json_response(...)`
   - `lasm_error_envelope(...)`
2. Added deterministic status-to-envelope mapping:
   - `400` -> `HTTP.BAD_REQUEST` / `validation`
   - `408` -> `HTTP.REQUEST_TIMEOUT` / `timeout`
   - `413` -> `HTTP.PAYLOAD_TOO_LARGE` / `resource_limit`
   - `417` -> `HTTP.EXPECTATION_FAILED` / `validation`
   - `431` -> `HTTP.REQUEST_HEADER_FIELDS_TOO_LARGE` / `resource_limit`
   - `501` -> `HTTP.NOT_IMPLEMENTED` / `internal`
   - `505` -> `HTTP.VERSION_NOT_SUPPORTED` / `validation`
3. Kept deterministic trace/header behavior intact (`X-Trace-Id`, status code, close-connection path).

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_method_token`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_header_encoding`
