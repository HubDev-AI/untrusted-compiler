# M38-S138 HTTP Max-Concurrency Throttle Drain-Timeout Whitespace-Token Fallback Hardening

## What it is

M38-S138 adds deterministic fallback coverage for whitespace-padded throttle drain-timeout values.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Whitespace-padded env values are common operator input artifacts and should not produce ambiguous runtime behavior.

## How it works

- Added:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_whitespace_env_falls_back_to_default_when_clang_available`
- The test sets `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS="20 "` and locks deterministic route-success fallback behavior.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_drain_timeout_whitespace_env_falls_back_to_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

1. Lock empty-token timeout fallback behavior (`M38-S139`).
