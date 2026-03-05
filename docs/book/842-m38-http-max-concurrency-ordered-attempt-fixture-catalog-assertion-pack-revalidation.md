# M38-S215 HTTP Max-Concurrency Ordered-Attempt Fixture Catalog Assertion-Pack Revalidation

## What it is

M38-S215 adds one-step assertion-pack helpers that combine fixture-catalog lookup and ordered contention assertion execution.

## Why it exists

S214 centralized fixture dispatch, but call sites still repeated nested `assert_*_branch_case(..., &*_branch_fixture(...))` wrappers.

## How it works

- Added assertion-pack helpers:
  - `assert_pair_contention_fixture(...)`
  - `assert_late_contention_fixture(...)`
  - `assert_burst_contention_fixture(...)`
- Each helper resolves the fixture catalog entry, then delegates to the existing branch-case assertion helper.
- Migrated all seven queue/security/late/burst ordered branches to one-step assertion-pack calls.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
