# M38-S224 HTTP Max-Concurrency Ordered-Attempt Descriptor Contract/Smoke Dual-Table Unification

## What it is

M38-S224 unifies ordered descriptor smoke and sanity coverage on one canonical expectation catalog.

## Why it exists

S223 compacted smoke coverage into a descriptor table, but descriptor contracts and smoke descriptors were still tracked in separate tables, which could drift.

## How it works

- Removed smoke-only descriptor table:
  - `ORDERED_CONTENTION_FIXTURE_SMOKE_DESCRIPTORS`
- Updated smoke harness:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
  to iterate `ORDERED_CONTENTION_RUNNER_METADATA_EXPECTATIONS` directly and execute each fixture descriptor.
- Kept pair/late/burst runtime behavior unchanged by reusing existing `run_ordered_contention_fixture_descriptor_case(...)`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
