# M38-S140 HTTP Max-Concurrency Throttle Drain-Timeout Malformed-Token Fallback Hardening

## What it is

M38-S140 adds deterministic fallback coverage for malformed timeout tokens (`20ms`, etc.).

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Malformed numeric-like tokens can bypass weak parsers. We lock deterministic fallback semantics for this path.

## How it works

- Added:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_malformed_env_falls_back_to_default_when_clang_available`
- The test sets `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS="20ms"` and asserts safe fallback route-success contract.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_drain_timeout_malformed_env_falls_back_to_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

1. Lock queue-boundary low-timeout contention contract (`M38-S141`).
