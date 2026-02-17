# M38-S225 HTTP Max-Concurrency Ordered-Attempt Descriptor Contract/Smoke Iterator-Helper Extraction

## What it is

M38-S225 extracts shared iterator helpers for ordered descriptor contract and smoke traversal.

## Why it exists

After S224, sanity and smoke harnesses both depended on the same expectation table but still reimplemented traversal loops inline.

## How it works

- Added shared expectation iterator helper:
  - `for_each_ordered_contention_runner_metadata_expectation(...)`
- Added descriptor projection iterator helper:
  - `for_each_ordered_contention_fixture_descriptor(...)`
- Migrated tests to helper traversal:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
