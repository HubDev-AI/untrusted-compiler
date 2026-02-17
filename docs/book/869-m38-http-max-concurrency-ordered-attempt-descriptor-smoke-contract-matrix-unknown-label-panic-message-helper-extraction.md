# M38-S242 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic-Message Helper Extraction

## What it is

M38-S242 extracts unknown matrix case-label panic message rendering into a dedicated helper and tightens panic-message contract assertions.

## Why it exists

After S241 extracted unknown-label assertion checks, panic-message text assembly still lived inline in resolver logic, leaving message contract assembly duplicated.

## How it works

- Added panic-message helper:
  - `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message(...)`
- Updated matrix case-label resolver:
  - `ordered_contention_smoke_contract_assertion_matrix_case(...)`
  to panic using the helper output.
- Updated unknown-label assertion helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic(...)`
  to assert exact deterministic message equality against helper output.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
