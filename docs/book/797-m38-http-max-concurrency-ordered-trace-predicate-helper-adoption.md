# M38-S170 HTTP Max-Concurrency Ordered-Trace Predicate Helper Adoption

## What it is

M38-S170 introduces ordered trace predicates for late and burst contention assertions.

## Why it exists

To avoid duplicating `rt-1/rt-2/rt-3` trace ordering assertions inline.

## How it works

- `late_ordered_trace_contract_holds(...)` validates `rt-1 success -> rt-2 throttle`.
- `burst_ordered_trace_contract_holds(...)` validates `rt-1 success -> rt-2/rt-3 throttle`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_`
