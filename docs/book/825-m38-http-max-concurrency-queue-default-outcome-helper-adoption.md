# M38-S198 HTTP Max-Concurrency Queue Default Outcome Helper Adoption

## What it is

M38-S198 migrates queue default contention branch to pair outcome helpers.

## Why it exists

To eliminate duplicated queue default outcome handling logic.

## How it works

- Queue default path now uses pair outcome collector, contract dispatch, and observation helper.
- Queue default one-success/one-throttle contract remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
