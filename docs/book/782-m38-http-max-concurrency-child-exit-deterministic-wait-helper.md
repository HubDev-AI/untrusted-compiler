# M38-S155 HTTP Max-Concurrency Child-Exit Deterministic Wait Helper

## What it is

M38-S155 introduces a bounded child-exit polling helper for contention tests.

## Why it exists

To remove duplicated wait loops and keep timeout fallback behavior consistent.

## How it works

- `wait_for_child_exit_or_terminate(...)` centralizes poll attempts, sleep cadence, and timeout kill+wait fallback.
- Contention tests use helper-specific timeout messages to retain case-local diagnostics.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
