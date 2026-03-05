# M38-S214 HTTP Max-Concurrency Ordered-Attempt Helper Fixture-Catalog Dispatch Normalization

## What it is

M38-S214 introduces fixture-catalog enums and dispatch helpers for ordered max-concurrency branch assertions.

## Why it exists

S213 removed field-level boilerplate, but per-test constructor argument lists still repeated long deterministic fixture payloads.

## How it works

- Added fixture enums:
  - `PairContentionBranchFixture`
  - `LateContentionBranchFixture`
  - `BurstContentionBranchFixture`
- Added dispatch helpers:
  - `pair_contention_branch_fixture(...)`
  - `late_contention_branch_fixture(...)`
  - `burst_contention_branch_fixture(...)`
- Migrated queue boundary/security parity/late/burst branch assertions to fixture-catalog lookups.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
