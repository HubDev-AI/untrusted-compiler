# M38-S232 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Banner Field-Key Constant Extraction

## What it is

M38-S232 extracts ordered descriptor smoke banner field keys into shared constants.

## Why it exists

After S231 formatter extraction, the banner field names were still inline literals inside formatter and contract expectations, leaving avoidable drift risk.

## How it works

- Added canonical banner field-key constants:
  - `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_CASE`
  - `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_MODULE`
  - `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_FIXTURE`
- Updated `ordered_contention_smoke_banner_formatter(...)` to compose banner output with field-key constants.
- Updated banner-contract expected formatting to use the same field-key constants.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
