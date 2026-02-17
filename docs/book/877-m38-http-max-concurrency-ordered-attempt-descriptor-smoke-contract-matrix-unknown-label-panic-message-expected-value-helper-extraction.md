# M38-S250 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic-Message Expected-Value Helper Extraction

## What it is

M38-S250 extracts unknown-label panic-message expected-value rendering into a dedicated helper.

## Why it exists

Expected panic-message computation was inline in the unknown-label panic-message assertion helper, which made expected-message contract behavior less reusable.

## How it works

- Added expected-message helper:
  - `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_expected_panic_message(...)`
- Migrated unknown-label panic-message assertion flow to consume the helper.
- Preserved deterministic expected-message rendering via the canonical panic-message formatter.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
