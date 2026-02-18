# M39 - LASM HEAD Body-Omission Parity

## Summary

Implemented explicit HTTP `HEAD` body-omission behavior in the LASM run backend response writer, including queue-overload (`503`) responses.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - Extended `write_lasm_http_response(...)` with an `omit_body` control.
   - Added request-method capture to overload-head parsing (`read_lasm_request_head`) so non-oneshot queue-overflow responses can treat `HEAD` requests correctly.
   - Threaded `HEAD` detection into runtime response writes and overload response writes.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_lasm_backend_overflow_head_omits_response_body`.
   - Test asserts:
     - overload status remains `503 Service Unavailable`,
     - deterministic JSON headers are preserved,
     - response body is omitted for `HEAD` overload requests.

## Why

This aligns LASM run behavior with HTTP semantics (`HEAD` should not write a response body) while keeping the deterministic overload JSON envelope contract for status/header observability.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_head_omits_response_body`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached`
