# M39 - LASM Conflicting Content-Length Rejection

## Summary

Hardened LASM inbound HTTP parsing to reject conflicting duplicate `Content-Length` headers with deterministic `400` diagnostics.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now tracks parsed content-length values.
   - When duplicate `Content-Length` headers disagree, parser now fails deterministically:
     - status: `400`
     - message: `conflicting content-length headers`

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`.

## Why

Conflicting content-length headers create ambiguous request framing and can lead to parser desynchronization. Rejecting this case deterministically hardens LASM request handling and keeps behavior predictable.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_501_for_transfer_encoding`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_supports_pipelined_requests_on_single_socket`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
