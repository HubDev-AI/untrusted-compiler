# M38-S182 HTTP Max-Concurrency Queue/Security Pair-Attempt Bootstrap Helper Normalization

## What it is

M38-S182 normalizes queue/security pair-attempt bootstrap setup behind shared helpers.

## Why it exists

To remove repeated attempt bootstrap logic in pair contention branches.

## How it works

- Queue/security branches now use dedicated pair bootstrap helpers for attempt setup.
- Existing deterministic contention contracts remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
