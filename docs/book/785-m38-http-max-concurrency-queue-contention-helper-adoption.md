# M38-S158 HTTP Max-Concurrency Queue Contention Helper Adoption

## What it is

M38-S158 migrates queue-boundary contention tests to the canonical helper layer.

## Why it exists

To guarantee queue default/low-timeout branches enforce identical deterministic setup and assertion paths.

## How it works

- Queue tests now use helper-driven connect/retry, I/O writes, response reads, and child exit wait.
- Success/throttle envelope contracts are asserted through shared predicates.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_`
