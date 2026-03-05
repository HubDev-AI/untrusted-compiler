# M38-S221 HTTP Max-Concurrency Ordered-Attempt Descriptor Contract Catalog Sanity Coverage

## What it is

M38-S221 adds deterministic sanity coverage for ordered descriptor catalog contracts in max-concurrency fixture orchestration.

## Why it exists

After S220, descriptor metadata and runner dispatch were centralized, but there was no explicit static contract test proving catalog values stayed aligned.

## How it works

- Added `assert_ordered_contention_runner_metadata(...)` helper to validate:
  - fixture id
  - module name
  - case label
  - source payload selection
  - branch fixture mapping
- Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage` test:
  - covers all pair/late/burst ordered descriptors
  - asserts descriptor-to-runner metadata contract invariants
  - runs without clang dependency
- Revalidated runtime contention pack after sanity coverage addition.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
