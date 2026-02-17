# M38-S244 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Constant Extraction

## What it is

M38-S244 extracts the matrix unknown-label sentinel into a shared constant for ordered smoke contract coverage.

## Why it exists

Unknown-label panic coverage used a raw inline label string, which risked drift between assertions and resolver behavior.

## How it works

- Added unknown-label sentinel constant:
  - `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_UNKNOWN_MATRIX_CASE_LABEL`
- Updated matrix case-label contract coverage to use the shared unknown-label constant.
- Kept panic-message contract behavior unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
