# M39 - LASM Request-Line Version Hardening

## Summary

Hardened LASM HTTP request parsing to require an explicit HTTP version token, enforce request-head byte limits across the request-line plus all headers, and reject unsupported protocol versions deterministically.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now enforces request-line shape:
     - method,
     - path,
     - http version,
     - no trailing extra tokens.
   - The request-line length now contributes to the `max_header_bytes` limit, returning `431` before normal header parsing when the limit is exceeded.
     - `request headers exceed configured limit ({max_header_bytes} bytes)`
   - Missing version now returns deterministic `400` with message:
     - `invalid request line: missing http version`
   - Unsupported versions now return deterministic `505` with message:
     - `unsupported http version: <version>`
   - Added `505 => HTTP Version Not Supported` to LASM status-text mapping.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_400_for_missing_http_version`.
   - Added `run_command_oneshot_lasm_backend_returns_505_for_unsupported_http_version`.
   - Added `run_command_oneshot_lasm_backend_returns_431_when_request_line_exceeds_header_limit` for overlong request-line enforcement.

## Why

This prevents ambiguous request-line parsing behavior and keeps LASM runtime diagnostics deterministic for malformed/unsupported protocol inputs.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_missing_http_version`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_505_for_unsupported_http_version`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_431_when_request_line_exceeds_header_limit`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_head_omits_response_body`
