# M38-S213 HTTP Max-Concurrency Ordered-Attempt Helper Branch-Case Fixture-Constructor Normalization

## What it is

M38-S213 adds canonical constructor helpers for pair/late/burst branch-case fixtures in ordered max-concurrency contention tests.

## Why it exists

S212 centralized branch-case data in structs, but each test still repeated verbose field-by-field fixture construction.

## How it works

- Added constructor helpers:
  - `pair_contention_branch_case(...)`
  - `late_contention_branch_case(...)`
  - `burst_contention_branch_case(...)`
- Migrated seven queue/security/late/burst branches to constructor-backed fixture calls.
- Kept assertion-loop execution and deterministic `last_observation` failure envelopes unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
