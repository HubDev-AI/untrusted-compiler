# M38-S130 HTTP Max-Concurrency Burst-Ingress Trace/Order Deterministic Coverage

## What it is

M38-S130 adds deterministic runtime e2e coverage for burst-ingress contention under max-concurrency in oneshot mode.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S129 covered one late-connection scenario (two clients). We still needed explicit proof that trace/order invariants stay deterministic when contention shape grows to a burst with multiple backlog clients.

This slice extends coverage to a three-client burst so runtime behavior under higher ingress pressure is pinned by contract.

## How it works

1. Added new runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
2. Test setup:
   - builds minimal `/health` `c-bin` fixture,
   - runs oneshot runtime with `SEC4_RT_HTTP_MAX_CONCURRENCY=1`,
   - connects first client (accepted path),
   - waits for accept/recv-block window,
   - connects second and third clients (late backlog burst),
   - writes second+third requests first, then first request to release the served path.
3. Deterministic assertions:
   - first response:
     - `HTTP/1.1 200 OK`
     - `X-Trace-Id: rt-1`
   - second response:
     - `HTTP/1.1 503 Service Unavailable`
     - `X-Trace-Id: rt-2`
   - third response:
     - `HTTP/1.1 503 Service Unavailable`
     - `X-Trace-Id: rt-3`
   - both throttled responses must include deterministic framing headers:
     - `Content-Type: text/plain; charset=utf-8`
     - `Content-Length: 37`
     - `Connection: close`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Under burst socket-close timing, throttle body visibility at client read boundary is not always stable even when response framing is deterministic; this slice intentionally locks deterministic status/trace/framing invariants first.
- Runtime algorithm changes are not part of this slice; this is deterministic coverage expansion.

## Next

1. Harden deterministic throttle body-delivery semantics under close-timing pressure (`M38-S131`).
2. Keep contention tests stable while tightening payload-level guarantees.
