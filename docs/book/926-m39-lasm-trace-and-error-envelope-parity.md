# M39 - LASM Trace And Error Envelope Parity

## Summary

Aligned LASM dynamic validation error responses with benchmark-style error envelope metadata by threading one per-request trace id through LASM run handling and emitting `traceId` + `timeMs` inside JSON error bodies.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `process_lasm_connection_with_runtime(...)` now allocates one trace id at request start and reuses it for:
     - response header `X-Trace-Id`
     - dynamic benchmark error envelopes.
   - Added helper `set_lasm_trace_id(...)` and reused it in the existing stamp path.
   - Extended `lasm_error_envelope(...)` to include:
     - `traceId`
     - `timeMs` (unix milliseconds via `lasm_now_ms()`).
   - Passed trace id into `apply_lasm_dynamic_response_materialization(...)`.

2. `compiler/sec4-cli/tests/commands.rs`
   - Extended invalid-payload LASM integration tests to assert error-envelope metadata:
     - `"traceId":"rt-1"`
     - `"timeMs":...`

## Why

For LASM benchmark parity, headers and body diagnostics should agree on request identity. This change makes LASM validation errors easier to correlate and closer to existing benchmark implementation envelope shape.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_json_payload`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_user_payload`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
