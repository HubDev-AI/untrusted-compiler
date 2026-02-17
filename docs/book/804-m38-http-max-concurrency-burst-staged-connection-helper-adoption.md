# M38-S177 HTTP Max-Concurrency Burst Staged-Connection Helper Adoption

## What it is

M38-S177 migrates burst contention setup to staged first/second/third connection helpers.

## Why it exists

To eliminate repeated staged connection setup code in burst default/low-timeout tests.

## How it works

- Burst default and low-timeout tests now call staged first/second/third connection helpers.
- Burst ordered contention contracts are unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_`
