# M38-S234 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Failure-Suffix Contract-Helper Extraction

## What it is

M38-S234 introduces a dedicated helper for ordered descriptor smoke failure-banner contract expected rendering.

## Why it exists

After S233 extracted the failure suffix constant, failure contract expected rendering still appeared inline in tests and separately in formatter logic.

## How it works

- Added helper:
  - `ordered_contention_smoke_failure_banner_contract_expected(...)`
- Migrated failure formatter:
  - `ordered_contention_smoke_failure_banner(...)`
  to delegate to the contract helper.
- Migrated failure contract coverage:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
  expected rendering to the same helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
