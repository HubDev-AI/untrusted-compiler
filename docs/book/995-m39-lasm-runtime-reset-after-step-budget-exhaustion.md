# M39: LASM Runtime Reset After Step-Budget Exhaustion

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- Added `LasmHttpRuntime::reset_transient_state()` in `sec4-core` to clear runtime transient execution state while preserving route registrations/configuration.
- Wired LASM `run --backend lasm` connection loop to call this reset path immediately after deterministic step-budget exhaustion responses.
- This clears in-flight tasks, pending queues, and ready-response carryover before processing subsequent requests on the same keep-alive connection.

## Why

Without runtime reset, a step-budget failure could leave unfinished tasks and queued responses alive inside the runtime, which could bleed into later requests and accumulate stale state.

## Validation

- `cargo test -p sec4-core reset_transient_state_clears_active_runtime_queues`
- `cargo test -p sec4 --test commands run_command_lasm_backend_counts_runtime_step_failures_toward_keep_alive_limit`
