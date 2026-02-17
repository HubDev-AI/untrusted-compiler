# M38-S212 HTTP Max-Concurrency Ordered-Attempt Helper Branch-Scope Contract Matrix Expansion

## What it is

M38-S212 expands ordered-attempt helper coverage by introducing explicit branch-case structs and assertion wrappers for queue/late/burst max-concurrency contention paths.

## Why it exists

The same ordered helper loop wiring was duplicated across seven branch tests, which made branch-scope contract updates noisy and easy to drift.

## How it works

- Added branch-case envelopes:
  - `PairContentionBranchCase`
  - `LateContentionBranchCase`
  - `BurstContentionBranchCase`
- Added branch-case assertion wrappers:
  - `assert_pair_contention_branch_case(...)`
  - `assert_late_contention_branch_case(...)`
  - `assert_burst_contention_branch_case(...)`
- Migrated queue boundary, security parity, late default/low-timeout, and burst default/low-timeout tests to branch-case matrix calls.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
