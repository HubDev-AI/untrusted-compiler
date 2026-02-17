# M38-S171 HTTP Max-Concurrency Bounded-Tail Latency Helper Normalization

## What it is

M38-S171 normalizes bounded-tail latency checks behind a shared helper.

## Why it exists

To keep timeout-window assertions aligned across queue, late, and burst contention branches.

## How it works

- `has_bounded_tail_latency(...)` takes attempt start time and max milliseconds.
- Low-timeout/default bounded-tail assertions now use the helper path.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
