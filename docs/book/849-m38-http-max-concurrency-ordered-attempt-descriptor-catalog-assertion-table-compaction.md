# M38-S222 HTTP Max-Concurrency Ordered-Attempt Descriptor Catalog Assertion-Table Compaction

## What it is

M38-S222 converts descriptor catalog sanity coverage from repeated assertion blocks into one table-driven expectation harness.

## Why it exists

S221 introduced deterministic sanity checks, but the test body repeated seven explicit assertion calls with mostly static metadata values.

## How it works

- Added `OrderedContentionRunnerMetadataExpectation` row type.
- Added canonical expectation table:
  - `ORDERED_CONTENTION_RUNNER_METADATA_EXPECTATIONS`
- Migrated `c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage` to iterate the expectation table and call `assert_ordered_contention_runner_metadata(...)`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
