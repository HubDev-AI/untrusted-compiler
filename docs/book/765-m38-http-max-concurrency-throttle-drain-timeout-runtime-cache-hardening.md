# M38-S142 HTTP Max-Concurrency Throttle Drain-Timeout Runtime Cache Hardening

## What it is

M38-S142 caches throttle drain-timeout once per `http.serve` lifecycle and reuses it across throttle close paths.

## Why it exists

Per-socket env parsing in close paths is unnecessary and can drift between paths. Serve-scoped caching makes behavior simpler and consistent.

## How it works

- `sec4_rt_http_serve(...)` now computes `throttle_drain_timeout_ms` once.
- Cached value is passed into:
  - `sec4_rt_drain_oneshot_backlog_with_throttle(...)`
  - queue-overflow close path
  - `sec4_rt_finalize_throttle_socket_close(...)`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Next

- Continue fallback-matrix/helper consolidation (`M38-S143`).
