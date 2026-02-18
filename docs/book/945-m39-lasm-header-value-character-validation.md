# M39: LASM Header Value Character Validation

## Why

LASM parser hardening already rejected invalid method/header-name tokens and malformed request targets, but header values still accepted control characters.

That left a protocol-validation gap where malformed header-value bytes could reach runtime handling paths.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added `is_lasm_http_header_value(...)` to enforce header-value character validity.
2. Shared request-head parser now rejects control characters in header values with deterministic diagnostics:
   - status: `400 Bad Request`
   - message: `invalid header line: invalid header value character`
3. Because validation lives in shared head parsing, behavior is consistent for:
   - normal LASM worker request parsing,
   - overflow-path head parsing.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_header_value_character`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_header_encoding`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request`
