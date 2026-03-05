# M39 - LASM HTTP Runtime Timeout From Submit Time

## What Was Added

Adjusted LASM HTTP timeout semantics so timeout age starts at request submit time (not only when task execution begins), which includes pending-queue wait in timeout evaluation.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Pending requests now retain `request_started_at_ms` captured at submit time.
2. When pending requests are drained, timed-out requests emit deterministic timeout responses immediately (`504`, `handler timed out after <limit>ms`) instead of starting a task that is already expired.
3. In-flight timeout checks continue to use the same deterministic timeout envelope, but now share submit-time origin.
4. `set_max_request_duration_ms(...)` now applies timeout checks immediately to current runtime state (in-flight cancellation + pending drain pass).

## Why

Timeout budgets should represent total request age under load, not only handler execution time. Counting queue wait avoids late task starts for already-expired requests and keeps async overload behavior deterministic.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`

New coverage:

- `pending_request_timeout_is_evaluated_from_submit_time`
