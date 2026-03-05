# M39 - LASM HTTP Runtime Request Timeout Enforcement

## What Was Added

Added deterministic request-timeout enforcement to LASM HTTP runtime, using scheduler cancellation primitives for live timed-out tasks.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Runtime now supports:
   - `set_max_request_duration_ms(limit_ms)`
   - `clear_max_request_duration_ms()`
2. `set_max_request_duration_ms(0)` is rejected deterministically (`must be >= 1ms`).
3. Requests exceeding configured duration produce deterministic timeout responses:
   - status `504`
   - body `handler timed out after <limit>ms`
4. Timeout mapping applies to:
   - handlers that complete after timeout threshold
   - live handlers cancelled after threshold is exceeded.

## Why

This moves LASM async runtime beyond queue-only flow control by adding deterministic per-request time budget enforcement, a core building block for resilient server-mode execution.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`

New coverage:

- `set_max_request_duration_rejects_zero`
- `completed_request_exceeding_timeout_maps_to_504`
- `live_request_exceeding_timeout_is_cancelled_and_emits_504`
