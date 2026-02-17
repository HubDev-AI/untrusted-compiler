# M38-S141 HTTP Max-Concurrency Queue-Boundary Low-Timeout Deterministic Throttle Contract Hardening

## What it is

M38-S141 adds low-timeout queue-boundary contention coverage for throttle body delivery and bounded tail behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Low-timeout contention coverage existed for late/burst paths; queue-boundary path needed the same minimum-budget guarantees.

## How it works

- Added:
  - `c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- Test config:
  - oneshot mode
  - `SEC4_RT_HTTP_MAX_CONCURRENCY=1`
  - `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=1`
  - trailing input noise on both clients
- Locks deterministic one-success/one-throttle contract with full throttle payload and bounded tail latency.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Next

1. Consolidate timeout-fallback matrix helpers/fixtures while preserving runtime contracts (`M38-S142`).
