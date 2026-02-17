# M38-S132 HTTP Max-Concurrency Throttle Close-Drain Timeout-Budget Coverage Hardening

## What it is

M38-S132 hardens max-concurrency throttle close-drain behavior by replacing fixed-attempt draining with a deterministic timeout-budget model.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

M38-S131 established deterministic throttle body delivery with graceful close sequencing, but drain behavior still depended on fixed loop attempts.

A timeout-budget model is safer and easier to reason about:

- deterministic upper bound on close-tail work,
- explicit configuration surface,
- less sensitivity to peer socket pacing.

## How it works

1. Added timeout-budget parser:
   - `sec4_rt_http_throttle_drain_timeout_ms(...)`
2. Added env bridge:
   - `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS`
   - default: `20ms`
   - bounded max clamp: `1000ms`
3. Updated close helper:
   - `sec4_rt_finalize_throttle_socket_close(...)` now computes a deadline from `sec4_rt_time_now()` and drains until EOF/error or timeout budget exhaustion.
4. Coverage hardening:
   - late/burst oneshot contention tests now stage trailing input noise on throttled sockets and require bounded completion while preserving full throttle body delivery.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Timeout-budget draining introduces a small bounded close-tail overhead in throttle paths.
- Conservative default budget improves delivery determinism under contention at the cost of slightly later final close on noisy peers.

## Next

1. Add explicit low-timeout deterministic-body coverage (`M38-S134`) for tighter contract pinning under aggressive timeout settings.
