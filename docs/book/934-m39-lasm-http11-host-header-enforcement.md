# M39 - LASM HTTP/1.1 Host Header Enforcement

## Summary

Added deterministic HTTP/1.1 `Host` header enforcement in the LASM request parser.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now validates that HTTP/1.1 requests include a non-empty `Host` header.
   - Missing/empty host now returns deterministic parser error:
     - status: `400`
     - message: `missing host header`

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_400_for_missing_host_header`.

## Why

HTTP/1.1 requests without host are malformed and can produce ambiguous routing/proxy behavior. Deterministic rejection keeps LASM parser behavior strict and predictable under malformed traffic.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_missing_host_header`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_supports_pipelined_requests_on_single_socket`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
