# M38-S233 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Failure-Suffix Constant Extraction

## What it is

M38-S233 extracts the ordered descriptor smoke failure suffix token into a shared constant.

## Why it exists

After S232 normalized banner field keys, failure-banner rendering still embedded the `failed:` token as an inline literal in formatter and contract expectations.

## How it works

- Added canonical failure suffix constant:
  - `ORDERED_CONTENTION_SMOKE_FAILURE_SUFFIX`
- Updated `ordered_contention_smoke_failure_banner(...)` to compose output with the shared suffix constant.
- Updated failure-banner contract expected rendering to use the same shared suffix constant.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
