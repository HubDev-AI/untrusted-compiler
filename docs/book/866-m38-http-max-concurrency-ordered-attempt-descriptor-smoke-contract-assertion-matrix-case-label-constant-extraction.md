# M38-S239 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Assertion-Matrix Case-Label Constant Extraction

## What it is

M38-S239 extracts ordered descriptor smoke assertion-matrix unknown-case panic prefix into a shared constant and adds explicit case-label contract coverage.

## Why it exists

After S238 introduced matrix case-label resolution, unknown-case panic text was still an inline literal and case-label behavior did not have dedicated contract tests.

## How it works

- Added unknown-case panic prefix constant:
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_UNKNOWN_MATRIX_CASE_LABEL_PREFIX`
- Updated matrix case-label resolver:
  - `ordered_contention_smoke_contract_assertion_matrix_case(...)`
  to render panic messages using the shared prefix constant.
- Added deterministic matrix case-label contract coverage test:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
  validating:
  - known case-label resolution (`case-banner`, `failure-banner`)
  - unknown case-label panic prefix + label inclusion.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
