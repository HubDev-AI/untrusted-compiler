# M38-S245 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Contract Helper Extraction

## What it is

M38-S245 extracts unknown-label matrix panic coverage behind a dedicated no-argument contract helper.

## Why it exists

Matrix case-label contract coverage previously invoked the unknown-label panic assertion directly, which made the test orchestration body noisier and harder to reuse.

## How it works

- Added unknown-label contract helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_contract()`
- Migrated matrix case-label contract coverage orchestration to call the helper.
- Preserved deterministic panic-message equality checks through the existing panic assertion helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
