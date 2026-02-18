# M39 - LASM Persistent Reader and Transfer-Encoding Rejection

## Summary

Hardened LASM worker HTTP handling by reusing one buffered request reader per connection, enabling deterministic pipelined parsing and preventing keep-alive read-ahead loss between requests. Also added deterministic rejection for unsupported transfer-encoding inputs.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `process_lasm_connection_with_runtime(...)` now creates one connection-scoped `BufReader<TcpStream>` and reuses it for all requests handled on the same keep-alive socket.
   - `read_lasm_http_request(...)` now reads from the persistent buffered reader instead of cloning a fresh stream per request.
   - Added deterministic unsupported transfer-encoding rejection:
     - status: `501`
     - message: `transfer-encoding is not supported`
   - Added deterministic comma-delimited `Connection` token parsing (`close`, `keep-alive`) instead of exact whole-header matching.
   - Added `501 -> Not Implemented` status-text mapping.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_lasm_backend_supports_pipelined_requests_on_single_socket`.
   - Added `run_command_oneshot_lasm_backend_returns_501_for_transfer_encoding`.

## Why

A fresh buffered reader per request can drop read-ahead bytes in keep-alive/pipelined workloads. Reusing one buffered reader per connection preserves parser correctness under real socket traffic and improves LASM worker behavior without changing deterministic runtime contracts.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_supports_pipelined_requests_on_single_socket`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_501_for_transfer_encoding`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_supports_keep_alive_for_multiple_requests`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
