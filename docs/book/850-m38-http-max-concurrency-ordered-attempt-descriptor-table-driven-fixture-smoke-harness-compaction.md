# M38-S223 HTTP Max-Concurrency Ordered-Attempt Descriptor Table-Driven Fixture Smoke Harness Compaction

## What it is

M38-S223 compacts ordered max-concurrency smoke coverage into one table-driven clang-gated harness.

## Why it exists

Even after S222 table-driven sanity assertions, runtime smoke coverage still repeated seven near-identical test bodies that differed only by fixture descriptor.

## How it works

- Added canonical smoke descriptor table:
  - `ORDERED_CONTENTION_FIXTURE_SMOKE_DESCRIPTORS`
- Added one clang-gated smoke harness test:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- Migrated queue/late/burst smoke execution to iterate the smoke descriptor table and call:
  - `run_ordered_contention_fixture_descriptor_case(...)`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
