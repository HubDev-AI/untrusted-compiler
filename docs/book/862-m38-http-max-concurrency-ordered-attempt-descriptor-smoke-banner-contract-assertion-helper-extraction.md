# M38-S235 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Banner-Contract Assertion-Helper Extraction

## What it is

M38-S235 extracts dedicated assertion helpers for ordered descriptor smoke case/failure banner contract coverage.

## Why it exists

After S234 centralized contract expected string assembly, banner contract tests still carried duplicated assertion blocks inline.

## How it works

- Added case-banner assertion helper:
  - `assert_ordered_contention_smoke_case_banner_contract(...)`
- Added failure-banner assertion helper:
  - `assert_ordered_contention_smoke_failure_banner_contract(...)`
- Updated contract tests to iterate descriptor expectations and dispatch helper assertions:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
