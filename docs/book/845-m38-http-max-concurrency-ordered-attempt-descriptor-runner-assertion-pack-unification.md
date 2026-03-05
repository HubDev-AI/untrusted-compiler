# M38-S218 HTTP Max-Concurrency Ordered-Attempt Descriptor-Runner Assertion-Pack Unification

## What it is

M38-S218 adds a unified dispatcher that routes ordered contention descriptor execution across pair, late, and burst branches.

## Why it exists

After S217, tests still selected runner families via separate entrypoints (`run_pair_*`, `run_late_*`, `run_burst_*`) at each call site.

## How it works

- Added `OrderedContentionFixtureDescriptor`:
  - `Pair(PairContentionFixtureDescriptor)`
  - `Late(LateContentionFixtureDescriptor)`
  - `Burst(BurstContentionFixtureDescriptor)`
- Added `run_ordered_contention_fixture_descriptor_case(...)` unified dispatcher.
- Migrated all seven ordered branch tests to this single dispatcher path.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
