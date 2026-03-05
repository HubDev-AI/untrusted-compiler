# M38-S237 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract-Helper Dispatch-Consolidator Extraction

## What it is

M38-S237 extracts a shared assertion-dispatch helper for ordered descriptor smoke contract coverage tests.

## Why it exists

After S236 helper naming cleanup, both case and failure banner contract tests still duplicated expectation-iteration dispatch logic.

## How it works

- Added shared contract assertion dispatcher:
  - `run_ordered_contention_smoke_contract_assertion_dispatch(...)`
- Updated case banner contract coverage test to call shared dispatcher with:
  - `assert_ordered_contention_smoke_case_banner_contract_expectation`
- Updated failure banner contract coverage test to call shared dispatcher with:
  - `assert_ordered_contention_smoke_failure_banner_contract_expectation`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
