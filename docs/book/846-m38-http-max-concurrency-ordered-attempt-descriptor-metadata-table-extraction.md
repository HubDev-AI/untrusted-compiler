# M38-S219 HTTP Max-Concurrency Ordered-Attempt Descriptor Metadata Table Extraction

## What it is

M38-S219 replaces descriptor metadata tuple `match` literals with canonical pair/late/burst static metadata tables.

## Why it exists

Descriptor catalogs in S217 still duplicated fixture metadata inline in three separate `match` helpers.

## How it works

- Added descriptor metadata structs:
  - `PairContentionFixtureMetadata`
  - `LateContentionFixtureMetadata`
  - `BurstContentionFixtureMetadata`
- Added static metadata tables:
  - `PAIR_CONTENTION_FIXTURE_METADATA_TABLE`
  - `LATE_CONTENTION_FIXTURE_METADATA_TABLE`
  - `BURST_CONTENTION_FIXTURE_METADATA_TABLE`
- Added deterministic descriptor index helpers:
  - `PairContentionFixtureDescriptor::as_index()`
  - `LateContentionFixtureDescriptor::as_index()`
  - `BurstContentionFixtureDescriptor::as_index()`
- Migrated descriptor lookup helpers to table extraction via `as_index`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
