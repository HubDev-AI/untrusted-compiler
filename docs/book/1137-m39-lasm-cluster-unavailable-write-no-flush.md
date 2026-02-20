# 1137 M39 Slice: LASM Cluster Unavailable Write Without Flush

This slice removes an extra syscall from LASM cluster unavailable response handling.

## What changed

1. Updated `write_lasm_cluster_unavailable_response(...)` to return after `write_all(...)`.
2. Removed explicit `TcpStream::flush()` call from relay unavailable response path.
3. Kept static prebuilt unavailable payloads and status/body contracts unchanged.

## Why

Unavailable responses already use connection-close semantics with prebuilt payload bytes.

An explicit flush per unavailable response adds extra syscall overhead in failure-heavy intervals without adding meaningful value for this close-after-write path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
