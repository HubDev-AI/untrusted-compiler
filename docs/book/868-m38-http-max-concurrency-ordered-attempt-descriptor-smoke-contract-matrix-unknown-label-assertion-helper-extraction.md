# M38-S241 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Assertion-Helper Extraction

## What it is

M38-S241 extracts unknown matrix case-label panic assertions into a dedicated helper for ordered smoke contract matrix coverage.

## Why it exists

After S240 extracted matrix case-label resolution helpers, unknown-label panic contract checks still lived inline inside the matrix case-label contract test.

## How it works

- Added helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic(...)`
- Migrated matrix case-label contract test:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
  to call the shared unknown-label assertion helper instead of inline catch-unwind/panic-message checks.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
