# M38-S135 HTTP Max-Concurrency Throttle Close-Drain Zero-Timeout Fallback Hardening

## What it is

M38-S135 locks deterministic fallback behavior when throttle close-drain timeout is configured to zero.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S133 covered invalid and over-cap timeout values, but zero-timeout configuration required explicit contract pinning.

A zero value must not silently collapse drain budget to non-deterministic behavior; it should map to safe default semantics.

## How it works

1. Added dedicated runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_zero_env_falls_back_to_default_when_clang_available`
2. Test enforces:
   - runtime remains healthy in oneshot mode,
   - route response contract stays deterministic (`200 OK` + `ok` body),
   - zero timeout is treated as fallback-safe configuration path.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_drain_timeout_zero_env_falls_back_to_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- This slice validates fallback behavior through deterministic e2e outcomes, not internal timer introspection.
- Minimum-budget behavior with non-fallback small values is deferred to `M38-S136`.

## Next

1. Harden and cover minimum-budget deterministic latency/contract behavior (`M38-S136`).
