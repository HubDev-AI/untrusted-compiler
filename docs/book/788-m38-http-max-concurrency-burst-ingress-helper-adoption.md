# M38-S161 HTTP Max-Concurrency Burst-Ingress Helper Adoption

## What it is

M38-S161 migrates burst-ingress contention tests (default + low-timeout) to the canonical helper layer.

## Why it exists

To keep three-connection ordered trace/body contracts deterministic and guard against helper drift.

## How it works

- Burst tests now use helper-driven connect/I/O/wait paths.
- Ordered trace assertions (`rt-1` success, `rt-2/rt-3` throttle) use shared trace-aware predicates.
- Attempt diagnostics use the canonical three-response formatter.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_`
