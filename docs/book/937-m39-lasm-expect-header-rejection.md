# M39 - LASM Expect Header Rejection

## Summary

Added deterministic LASM parser rejection for unsupported `Expect` request headers.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now rejects non-empty `Expect` headers.
   - Deterministic parser response:
     - status: `417`
     - message: `expect header is not supported`
   - Added status text mapping for `417 Expectation Failed`.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_417_for_expect_header`.

## Why

`Expect` negotiation (for example `100-continue`) is not implemented in LASM runtime. Deterministic rejection avoids partial/ambiguous request-body behavior and keeps protocol handling explicit.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_417_for_expect_header`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_501_for_transfer_encoding`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
