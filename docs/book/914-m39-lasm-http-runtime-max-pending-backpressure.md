# M39 - LASM HTTP Runtime Max Pending Backpressure

## What Was Added

Added deterministic pending-queue backpressure to LASM HTTP runtime and surfaced it through `sec4 lasm-smoke`.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`
2. `compiler/sec4-cli/src/main.rs`
3. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. LASM runtime now supports:
   - `set_max_pending(limit)`
   - `clear_max_pending()`
2. `set_max_pending(0)` is rejected deterministically (`must be >= 1`).
3. When in-flight slots are full and pending queue reaches limit, new requests are rejected immediately with:
   - status `503`
   - body `runtime queue full`
4. Queue-overflow responses preserve resolved path params for observability.
5. `sec4 lasm-smoke` now supports:
   - `--max-pending <N>`
6. Smoke summaries now include `maxPending` (text/json).
7. Smoke summaries now include deterministic `statusCounts` to show mixed success/error distributions under queue pressure.
8. `sec4 lasm-smoke` now supports `--fail-on-errors` to turn observed error responses (for example queue overflow `503`) into deterministic command failure.

## Why

This introduces real async overload behavior for LASM bootstrap: bounded queueing with deterministic overflow response instead of unbounded pending growth.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`

New/updated coverage highlights:

- `set_max_pending_rejects_zero`
- `queue_overflow_returns_deterministic_503_response`
- `lasm_smoke_command_rejects_zero_max_pending`
- `lasm_smoke_command_reports_queue_overflow_with_max_pending_limit`
