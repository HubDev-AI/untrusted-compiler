# M38-S133 HTTP Max-Concurrency Throttle Drain-Timeout Env Fallback/Clamp Hardening

## What it is

M38-S133 adds deterministic fallback and clamp coverage for throttle drain-timeout runtime configuration.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

After introducing `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS`, we need contract coverage proving runtime stays safe when operators provide invalid or extreme values.

Without this, timeout-budget behavior could regress silently through env parsing drift.

## How it works

1. Extended max-concurrency env helper coverage to include throttle drain-timeout env input.
2. Added invalid-value fallback test:
   - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_invalid_env_falls_back_to_default_when_clang_available`
3. Added over-cap clamp test:
   - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_over_cap_env_is_clamped_when_clang_available`
4. Both tests lock route-success contract (`200 OK` + body) under deterministic env handling.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- These tests validate configuration behavior through runtime e2e contract outcomes rather than internal timing probes.
- Detailed low-timeout body-delivery stress coverage is deferred to the next slice (`M38-S134`).

## Next

1. Pin explicit low-timeout deterministic throttle body-delivery contracts in contention paths (`M38-S134`).
