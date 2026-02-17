# M38-S184 HTTP Max-Concurrency Queue Default Pair-Attempt Spawn Helper Adoption

## What it is

M38-S184 migrates queue default contention attempt bootstrap to the pair-attempt spawn helper.

## Why it exists

To remove duplicated queue default attempt bootstrap code.

## How it works

- Queue default branch now uses `spawn_pair_contention_attempt(...)`.
- Queue default one-success/one-throttle contract remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
