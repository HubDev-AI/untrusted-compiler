# M38-S134 HTTP Max-Concurrency Throttle Close-Drain Low-Timeout Deterministic Body Contract Coverage

## What it is

M38-S134 adds explicit low-timeout contention coverage for throttle close-drain behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S132 introduced timeout-budget close-drain semantics and M38-S133 locked env fallback/clamp behavior.

We still needed direct proof that aggressively low timeout configuration preserves:

- deterministic throttle body delivery,
- deterministic trace/order contracts,
- bounded close-tail runtime behavior.

## How it works

1. Added a new burst-ingress runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
2. Test setup:
   - oneshot mode, `SEC4_RT_HTTP_MAX_CONCURRENCY=1`,
   - low close-drain timeout: `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=1`,
   - staged trailing input noise on both throttled clients.
3. Deterministic assertions:
   - first response remains success (`rt-1` + `ok`),
   - second/third responses remain `503` throttle (`rt-2`, `rt-3`),
   - both throttled responses retain full payload body with `Content-Length: 37`,
   - bounded tail-latency stays within contract.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- The test increases contention-stress coverage depth and adds runtime e2e execution time in the max-concurrency suite.
- It favors stronger runtime contract guarantees over lighter test surface.

## Next

1. Harden and pin zero/near-zero timeout fallback semantics for throttle close-drain configuration (`M38-S135`).
