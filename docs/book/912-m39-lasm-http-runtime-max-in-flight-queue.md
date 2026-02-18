# M39 - LASM HTTP Runtime Max In-Flight Queue

## What Was Added

Added in-memory concurrency gating to LASM HTTP runtime with deterministic FIFO pending-request draining.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Runtime now supports optional in-flight concurrency caps via:
   - `set_max_in_flight(limit)`
   - `clear_max_in_flight()`
2. `set_max_in_flight(0)` is rejected deterministically (`must be >= 1`).
3. When capacity is full, route-resolved requests are queued in FIFO order instead of starting immediately.
4. Completed requests free capacity, and pending requests start deterministically in FIFO order.
5. Runtime run loop now continues scheduling newly-unblocked queued requests within the same `run_until_idle(max_steps)` budget window.
6. Runtime idle reporting now reflects pending queue and in-flight state so queued work cannot be reported as idle.

## Why

This adds real async-backend flow control behavior to LASM bootstrap without requiring full network-stack replacement yet, and provides deterministic backpressure semantics for future concurrency slices.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_runs_in_memory_runtime_with_compiled_entrypoint`

New/updated runtime coverage:

- `set_max_in_flight_rejects_zero`
- `max_in_flight_queues_pending_requests_fifo`
