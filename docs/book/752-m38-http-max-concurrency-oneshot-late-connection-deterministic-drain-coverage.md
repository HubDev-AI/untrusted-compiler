# M38-S129 HTTP Max-Concurrency Oneshot Late-Connection Deterministic Drain Coverage

## What it is

M38-S129 adds deterministic runtime e2e coverage for the oneshot late-connection drain path under max-concurrency pressure.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S128 hardened oneshot backlog draining, but we still needed explicit contract coverage for a specific race-prone path:

- first client accepted and blocked in handler recv,
- second client connects later (after accept-loop phase),
- runtime must still drain-throttle the late client deterministically after serving the first request.

Without this dedicated coverage, late-connection behavior could regress to nondeterministic close/no-response outcomes.

## How it works

1. Added new runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
2. Test setup:
   - builds minimal `/health` `c-bin` fixture,
   - runs runtime in oneshot mode with `SEC4_RT_HTTP_MAX_CONCURRENCY=1`,
   - establishes first connection,
   - waits briefly to allow first-connection accept + recv-block state,
   - establishes second (late) connection,
   - writes second request first, then first request to release handler progression.
3. Deterministic assertions:
   - first response is success:
     - `HTTP/1.1 200 OK`
     - `X-Trace-Id: rt-1`
     - body `ok`
   - second response is throttle:
     - `HTTP/1.1 503 Service Unavailable`
     - `X-Trace-Id: rt-2`
     - `Content-Type: text/plain; charset=utf-8`
     - `Connection: close`
     - body `server busy: max concurrency exceeded`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- The test intentionally uses timed staging (`sleep`) to force a realistic late-connection window; it balances determinism with practical runtime scheduling behavior.
- This slice is coverage hardening only; runtime algorithm changes were completed in M38-S128.

## Next

1. Expand burst-ingress contention coverage with stronger trace/order invariants across more than two clients (`M38-S130`).
2. Keep max-concurrency suite deterministic while increasing contention-shape diversity.
