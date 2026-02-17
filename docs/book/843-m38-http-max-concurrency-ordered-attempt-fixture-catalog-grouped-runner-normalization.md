# M38-S216 HTTP Max-Concurrency Ordered-Attempt Fixture-Catalog Grouped Runner Normalization

## What it is

M38-S216 introduces grouped runner helpers for pair/late/burst ordered max-concurrency fixture execution.

## Why it exists

Each ordered branch test still repeated local `build_c_bin_fixture(...)`, request bytes, and assertion invocation boilerplate.

## How it works

- Added grouped runner helpers:
  - `run_pair_contention_fixture_case(...)`
  - `run_late_contention_fixture_case(...)`
  - `run_burst_contention_fixture_case(...)`
- Added shared route fixture source constants:
  - `MAX_CONCURRENCY_SIMPLE_HEALTH_ROUTER_SOURCE`
  - `MAX_CONCURRENCY_SECURITY_HEADERS_ROUTER_SOURCE`
- Migrated seven queue/security/late/burst tests to grouped runner entrypoints.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
