# M38-S220 HTTP Max-Concurrency Ordered-Attempt Descriptor Table-Runner Envelope Simplification

## What it is

M38-S220 unifies ordered descriptor execution behind a single runner metadata envelope and dispatcher path.

## Why it exists

S219 still routed ordered descriptor execution through multiple runner helper layers before reaching build/assert.

## How it works

- Added `OrderedContentionBranchFixture` to represent pair/late/burst assertion branch dispatch.
- Added `OrderedContentionRunnerMetadata` as canonical runner envelope:
  - fixture name
  - module name
  - case label
  - source
  - branch fixture dispatch
- Added `ordered_contention_runner_metadata(...)` to resolve descriptors into one runner envelope.
- Simplified `run_ordered_contention_fixture_descriptor_case(...)` to:
  - resolve envelope
  - build fixture once
  - execute one canonical branch assertion dispatcher

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
