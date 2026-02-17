# M38-S139 HTTP Max-Concurrency Throttle Drain-Timeout Empty-Token Fallback Hardening

## What it is

M38-S139 adds deterministic fallback coverage for empty throttle drain-timeout env values.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Empty env values often appear in CI/environment templating and must map to safe defaults.

## How it works

- Added:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_empty_env_falls_back_to_default_when_clang_available`
- The test sets `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=""` and locks deterministic fallback route-success contract.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_drain_timeout_empty_env_falls_back_to_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

1. Lock malformed-token timeout fallback behavior (`M38-S140`).
