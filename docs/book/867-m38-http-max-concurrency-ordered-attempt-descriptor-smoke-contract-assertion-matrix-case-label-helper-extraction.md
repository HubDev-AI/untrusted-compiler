# M38-S240 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Assertion-Matrix Case-Label Helper Extraction

## What it is

M38-S240 extracts shared case-label helper functions for ordered descriptor smoke assertion-matrix dispatch and known-label resolution checks.

## Why it exists

After S239 introduced matrix case-label constants and contract coverage, tests still repeated matrix-case resolution/dispatch steps inline.

## How it works

- Added case-label dispatch helper:
  - `run_ordered_contention_smoke_contract_assertion_matrix_case_label(...)`
- Added known-label resolution assertion helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_case_label_resolution(...)`
- Migrated tests:
  - case/failure banner contract coverage uses the shared case-label dispatch helper
  - matrix case-label contract coverage uses shared known-label assertion helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
