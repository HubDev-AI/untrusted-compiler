# M38-S137 HTTP Max-Concurrency Throttle Drain-Timeout Negative-Value Fallback Hardening

## What it is

M38-S137 adds deterministic fallback coverage for negative throttle drain-timeout configuration values.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Timeout fallback matrix coverage included invalid/zero/over-cap values, but negative input needed explicit contract pinning.

Negative values are common operator mistakes and should never degrade runtime determinism.

## How it works

1. Added runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_negative_env_falls_back_to_default_when_clang_available`
2. Test config:
   - sets `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=-1`,
   - validates deterministic route-success contract (`200 OK` + body).
3. This locks negative-value behavior into safe fallback semantics.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_drain_timeout_negative_env_falls_back_to_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Coverage remains route-contract focused; it does not introspect internal timeout-state values directly.
- Keeps test surface small while pinning externally observable fallback guarantees.

## Next

1. Lock whitespace-token timeout fallback semantics (`M38-S138`).
