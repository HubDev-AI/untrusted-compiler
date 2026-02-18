# M39 - LASM Runtime Task Cancellation Baseline

## What Was Added

Added explicit task-cancellation support to the LASM async scheduler runtime.

Files:

1. `compiler/sec4-core/src/lasm_runtime.rs`

## Behavior

1. New runtime APIs:
   - `cancel_task(task_id) -> bool`
   - `live_task_count() -> usize`
2. Cancelling an existing task removes it from live scheduler state without producing a completion record.
3. Cancelling an unknown task returns `false` deterministically.
4. Ready-queue references to cancelled tasks are removed to avoid stale scheduling churn.

## Why

Cancellation is required groundwork for timeout and policy-enforced request abort behavior in higher-level LASM HTTP runtime slices.

## Validation

1. `cargo test -p sec4-core lasm_runtime::tests::`

New coverage:

- `cancel_task_removes_live_task_without_completion`
