# M38-S128 HTTP Max-Concurrency Throttle Security-Header Parity Hardening

## What it is

M38-S128 hardens runtime behavior so max-concurrency throttle responses preserve security-header parity when security middleware is enabled, including oneshot backlog-drain paths.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S127 locked deterministic success/throttle splitting under queue pressure, but it also exposed a parity gap risk: in oneshot contention windows, late/pending clients could be closed without deterministic throttle envelopes, making security-header parity checks brittle and behavior less predictable.

This slice closes that gap by forcing deterministic throttle responses for backlog drain paths and asserting header parity under contention.

## How it works

1. Added runtime helper:
   - `sec4_rt_drain_oneshot_backlog_with_throttle(...)`
2. Serve-loop behavior change:
   - in oneshot mode, after the first served request, runtime now drains:
     - already-pending accepted clients, and
     - newly accepted backlog sockets
   - each drained client receives deterministic max-concurrency `503` throttle response via `sec4_rt_send_concurrency_throttle_response(...)`.
3. Added e2e parity test:
   - `c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
   - builds a `c-bin` fixture with:
     - `sec.defaultHeaders()`
     - `sec.withSecurityHeaders(...)`
     - `SEC4_RT_HTTP_MAX_CONCURRENCY=1`
   - asserts one success + one throttle response and validates security headers on both response classes.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Backlog drain now prefers deterministic throttle responses over silent closes in oneshot contention tails, which improves observability and contract consistency at the cost of slightly more response work during shutdown.
- This slice does not change the core queueing algorithm or max-concurrency sizing logic; it hardens response semantics and parity only.

## Next

1. Add explicit late-connection coverage for oneshot contention tails (`M38-S129`).
2. Expand deterministic assertions for backlog-drain ordering and trace-envelope invariants under bursty ingress.
