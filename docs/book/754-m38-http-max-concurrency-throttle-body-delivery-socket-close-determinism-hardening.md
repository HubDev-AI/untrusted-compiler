# M38-S131 HTTP Max-Concurrency Throttle Body-Delivery Socket-Close Determinism Hardening

## What it is

M38-S131 hardens runtime throttle close behavior so max-concurrency `503` responses keep deterministic payload delivery under race-prone socket-close timing.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Earlier slices (`M38-S129`/`M38-S130`) proved deterministic status/trace/framing for late and burst oneshot contention, but body visibility could still be timing-sensitive in close races.

The runtime was closing throttled sockets immediately after write, which can produce reset-like behavior when peer input remains unread, even when `Content-Length` advertises a body.

This slice hardens close sequencing and restores strict body assertions in the race-heavy tests.

## How it works

1. Added runtime helper:
   - `sec4_rt_finalize_throttle_socket_close(...)`
2. Helper behavior:
   - performs `shutdown(fd, SHUT_WR)` to half-close the write side after throttle response emission,
   - switches socket to nonblocking mode,
   - runs bounded peer-input drain loops with short `select(...)` waits,
   - exits quickly on EOF/error/no-ready-input to avoid unbounded stall.
3. Wired helper into all max-concurrency throttle close call sites:
   - queue-boundary overflow throttle path,
   - oneshot pending backlog drain path,
   - oneshot newly accepted backlog drain path.
4. Tightened e2e contracts back to payload-required assertions:
   - late-connection test now requires full throttle body bytes,
   - burst-ingress test now requires full throttle body bytes on both throttled responses.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- The bounded drain loop adds small per-throttle close overhead (short micro-timeouts) in contention paths.
- The implementation prefers deterministic response integrity over immediate close speed in throttled paths.
- This slice does not change max-concurrency sizing or queue strategy; it hardens close semantics only.

## Next

1. Add `M38-S132` coverage to lock bounded drain-time behavior under heavier contention envelopes.
2. Keep max-concurrency contention contracts strict on payload + trace + framing invariants.
