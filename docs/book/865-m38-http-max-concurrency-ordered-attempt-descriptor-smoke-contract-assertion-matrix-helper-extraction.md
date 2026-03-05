# M38-S238 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Assertion-Matrix Helper Extraction

## What it is

M38-S238 adds a canonical assertion-matrix helper for ordered descriptor smoke banner contract coverage.

## Why it exists

After S237 consolidated assertion dispatch, contract tests still directly selected assertion helpers, without one explicit matrix that binds case labels to assertion functions.

## How it works

- Added assertion-matrix row type:
  - `OrderedContentionSmokeContractAssertionMatrixCase`
- Added canonical matrix + labels:
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_MATRIX`
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_CASE_BANNER_CASE_LABEL`
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_FAILURE_BANNER_CASE_LABEL`
- Added matrix-case resolver/executor helpers:
  - `ordered_contention_smoke_contract_assertion_matrix_case(...)`
  - `run_ordered_contention_smoke_contract_assertion_matrix_case(...)`
- Updated case/failure contract tests to resolve and execute matrix cases.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
