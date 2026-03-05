# M38-S136 HTTP Max-Concurrency Throttle Close-Drain Minimum-Budget Late-Connection Hardening

## What it is

M38-S136 adds explicit minimum-budget late-connection coverage for throttle close-drain behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S134 locked low-timeout burst behavior. We still needed a symmetric late-connection path contract for minimum-budget settings.

Without this, minimum-budget guarantees were stronger on burst ingress than on single late backlog drains.

## How it works

1. Added new runtime e2e test:
   - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
2. Test setup:
   - oneshot mode, `SEC4_RT_HTTP_MAX_CONCURRENCY=1`,
   - minimum timeout budget: `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=1`,
   - staged trailing input noise on late throttled client.
3. Deterministic assertions:
   - first response remains `200 OK` with `rt-1`,
   - late response remains `503` with `rt-2`,
   - throttle body payload remains present with `Content-Length: 37`,
   - tail latency remains bounded.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds another contention-heavy e2e test to max-concurrency suite runtime.
- Strengthens low-timeout contract confidence at the cost of slightly longer test runtime.

## Next

1. Lock negative-value timeout fallback behavior for throttle close-drain config (`M38-S137`).
