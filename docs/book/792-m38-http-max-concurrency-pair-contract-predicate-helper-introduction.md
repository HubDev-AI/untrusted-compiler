# M38-S165 HTTP Max-Concurrency Pair-Contract Predicate Helper Introduction

## What it is

M38-S165 introduces a pair contract predicate for one-success/one-throttle assertions.

## Why it exists

To keep pair contention assertions deterministic and free from duplicated branch logic.

## How it works

- `pair_success_throttle_contract_holds(...)` validates process success plus success/throttle envelopes.
- Queue contention paths consume this helper directly.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_`
