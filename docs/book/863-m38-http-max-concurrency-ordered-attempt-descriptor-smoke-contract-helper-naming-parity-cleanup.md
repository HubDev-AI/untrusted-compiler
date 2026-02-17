# M38-S236 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract-Helper Naming Parity Cleanup

## What it is

M38-S236 normalizes ordered descriptor smoke contract helper naming to consistent expectation-oriented names.

## Why it exists

After S235 extracted contract assertion helpers, helper names still mixed naming style and did not explicitly signal expectation-row scope.

## How it works

- Renamed case-banner contract helper:
  - `assert_ordered_contention_smoke_case_banner_contract(...)`
  - to `assert_ordered_contention_smoke_case_banner_contract_expectation(...)`
- Renamed failure-banner contract helper:
  - `assert_ordered_contention_smoke_failure_banner_contract(...)`
  - to `assert_ordered_contention_smoke_failure_banner_contract_expectation(...)`
- Updated case/failure banner contract coverage tests to dispatch through renamed helpers.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
