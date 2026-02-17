# M38-S243 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Case-Label Resolver Assertion-Helper Extraction

## What it is

M38-S243 extracts a dedicated case-label resolver assertion helper and a canonical known-label set for ordered smoke contract matrix coverage.

## Why it exists

After S242 tightened unknown-label panic contracts, known-label resolver checks were still written as direct assertions in the matrix contract test.

## How it works

- Added resolver assertion helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_case_label_resolver_contract(...)`
- Added canonical known-label set:
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_KNOWN_MATRIX_CASE_LABELS`
- Updated matrix case-label contract coverage test:
  - loops known labels through resolver assertion helper
  - keeps unknown-label panic assertions unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
