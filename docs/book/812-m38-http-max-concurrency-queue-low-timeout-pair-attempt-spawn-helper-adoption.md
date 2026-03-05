# M38-S185 HTTP Max-Concurrency Queue Low-Timeout Pair-Attempt Spawn Helper Adoption

## What it is

M38-S185 migrates queue low-timeout contention attempt bootstrap to the pair-attempt spawn helper.

## Why it exists

To remove duplicated queue low-timeout attempt bootstrap code.

## How it works

- Queue low-timeout branch now uses `spawn_pair_contention_attempt(...)`.
- Queue low-timeout bounded-tail contract remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
