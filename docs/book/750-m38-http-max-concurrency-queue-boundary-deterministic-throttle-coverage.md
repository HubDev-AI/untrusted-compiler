# M38-S127 HTTP Max-Concurrency Queue-Boundary Deterministic Throttle Coverage

## What it is

M38-S127 adds deterministic runtime e2e coverage for queue-boundary throttling when ingress pressure exceeds `SEC4_RT_HTTP_MAX_CONCURRENCY`.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Previous slices (`M38-S124` to `M38-S126`) established max-concurrency materialization, CLI precedence, and env fallback/clamp behavior.

The remaining gap was queue-boundary contention behavior itself: proving that concurrent accepted connections produce a deterministic success/throttle split with a stable `503` throttle envelope contract.

## How it works

1. Added runtime `c-bin` queue-pressure e2e test:
   - `c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
2. Test setup:
   - builds a minimal `/health` HTTP fixture,
   - runs runtime in oneshot mode,
   - enforces `SEC4_RT_HTTP_MAX_CONCURRENCY=1`,
   - opens two concurrent client connections and sends requests.
3. Contract assertions:
   - exactly one response is `HTTP/1.1 200 OK`,
   - exactly one response is `HTTP/1.1 503 Service Unavailable`,
   - throttle response contains deterministic envelope components:
     - `Content-Type: text/plain; charset=utf-8`
     - `Content-Length: 37`
     - `Connection: close`
     - `X-Trace-Id: rt-*`
     - body `server busy: max concurrency exceeded`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- This slice hardens queue-boundary behavior through runtime contract coverage; it does not redesign ingress scheduling.
- The test intentionally validates deterministic throttle framing and body/trace contract, while leaving security-header parity as the next focused slice.

## Next

1. Harden throttle-response security-header parity with normal/error response paths (`M38-S128`).
2. Add explicit assertions for throttle-path security-header propagation once parity is implemented.
