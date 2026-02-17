# M38-S180 HTTP Max-Concurrency Burst Staged-Attempt Spawn Helper Adoption

## What it is

M38-S180 migrates burst contention attempt bootstrap to the staged-attempt spawn helper.

## Why it exists

To remove duplicated burst-path bootstrap boilerplate.

## How it works

- Burst default and low-timeout tests now call `spawn_staged_contention_attempt(...)`.
- Burst runtime contention contracts are unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_`
