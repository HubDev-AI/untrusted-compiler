# M39 - LASM Worker Keep-Alive Support

## Summary

Added HTTP keep-alive support for LASM non-oneshot worker connections so a single socket can serve multiple requests sequentially.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `process_lasm_connection_with_runtime(...)` now supports connection reuse loops in worker mode.
   - Added `allow_keep_alive` control so oneshot mode still serves one request deterministically.
   - Added connection-close resolution rules:
     - explicit `Connection: close` closes,
     - explicit `Connection: keep-alive` keeps open,
     - HTTP/1.0 defaults to close.
   - `write_lasm_http_response(...)` now sets deterministic `Connection: keep-alive|close` headers based on runtime decision.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_lasm_backend_supports_keep_alive_for_multiple_requests`.
   - Test validates two sequential requests over one socket with expected connection-header transitions (`keep-alive` then `close`).

## Why

This improves LASM backend throughput behavior by reducing per-request socket setup in worker mode while preserving deterministic control of close behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_supports_keep_alive_for_multiple_requests`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_head_omits_response_body`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_505_for_unsupported_http_version`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
