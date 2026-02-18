# M39: LASM Keep-Alive Accounting On Runtime-Step Failure

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- `process_lasm_connection_with_runtime` now increments `responses_written` after emitting deterministic runtime step-budget failure responses (`500 Internal Server Error` path when runtime remains active after step budget).
- This keeps keep-alive accounting consistent with success and preflight/error branches.
- Added command integration coverage proving keep-alive limits still close the connection after repeated step-budget failures.

## Why

Before this fix, the step-budget failure branch skipped response-count increments. Under keep-alive connections, repeated runtime-step failures could bypass `max_keep_alive_requests` close enforcement.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_counts_runtime_step_failures_toward_keep_alive_limit`
- `cargo test -p sec4 --test commands run_command_lasm_backend_cli_keep_alive_override_takes_precedence_over_env`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_max_runtime_steps_from_policy`
