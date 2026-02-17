# M38-S252 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic-Message Equality Helper Extraction

## What it is

M38-S252 extracts unknown-label panic-message equality assertions into a dedicated helper.

## Why it exists

The panic-message assertion helper still inlined direct equality checks, which kept deterministic contract assertion logic coupled to message extraction.

## How it works

- Added equality assertion helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message_equality(...)`
- Migrated unknown-label panic-message assertion flow to delegate final equality assertions through the helper.
- Kept equality assertion text and deterministic behavior unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
