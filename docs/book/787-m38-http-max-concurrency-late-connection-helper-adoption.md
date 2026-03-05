# M38-S160 HTTP Max-Concurrency Late-Connection Helper Adoption

## What it is

M38-S160 migrates late-connection contention tests (default + low-timeout) to the canonical helper layer.

## Why it exists

To keep late-connection drain and bounded-tail contracts locked to one deterministic setup/assertion implementation.

## How it works

- Late tests use helper-driven connection retry/terminate and child wait logic.
- Ordered trace contract (`rt-1` success, `rt-2` throttle) now uses trace-aware shared predicates.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_`
