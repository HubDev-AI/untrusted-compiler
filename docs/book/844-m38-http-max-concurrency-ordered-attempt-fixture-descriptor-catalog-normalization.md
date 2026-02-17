# M38-S217 HTTP Max-Concurrency Ordered-Attempt Fixture Descriptor Catalog Normalization

## What it is

M38-S217 adds descriptor catalogs that map pair/late/burst fixture identifiers to canonical fixture metadata.

## Why it exists

S216 grouped runner helpers still required literal fixture metadata at test call sites, which duplicated fixture names/module names/labels.

## How it works

- Added descriptor enums:
  - `PairContentionFixtureDescriptor`
  - `LateContentionFixtureDescriptor`
  - `BurstContentionFixtureDescriptor`
- Added descriptor catalog helpers:
  - `pair_contention_fixture_descriptor(...)`
  - `late_contention_fixture_descriptor(...)`
  - `burst_contention_fixture_descriptor(...)`
- Added descriptor-driven runner entrypoints:
  - `run_pair_contention_fixture_descriptor_case(...)`
  - `run_late_contention_fixture_descriptor_case(...)`
  - `run_burst_contention_fixture_descriptor_case(...)`
- Migrated seven ordered branch tests to descriptor-driven grouped runner calls.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
