# M38-S143 HTTP Max-Concurrency Timeout Fallback-Matrix Assertion Helper Dedup

## What it is

M38-S143 introduces a shared assertion helper for timeout fallback/clamp route-success checks.

## Why it exists

Fallback matrix tests repeated identical route-success assertions with only case labels changing.

## How it works

- Added `assert_throttle_drain_timeout_env_route_success(...)`.
- Timeout matrix tests now call this helper with case-specific labels.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Deduplicate spawn wiring (`M38-S144`).
