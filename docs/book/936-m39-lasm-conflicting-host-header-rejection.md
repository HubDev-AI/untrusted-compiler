# M39 - LASM Conflicting Host Header Rejection

## Summary

Added deterministic rejection for conflicting duplicate `Host` headers in the LASM HTTP request parser.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now tracks parsed `Host` header values.
   - When duplicate `Host` headers disagree, parser fails deterministically:
     - status: `400`
     - message: `conflicting host headers`
   - HTTP/1.1 host-presence enforcement now uses parsed host-header state.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_400_for_conflicting_host_headers`.

## Why

Conflicting duplicate host headers create ambiguous authority resolution and can be abused in request-routing edge cases. Deterministic rejection keeps LASM parser behavior strict and predictable.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_host_headers`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_missing_host_header`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
