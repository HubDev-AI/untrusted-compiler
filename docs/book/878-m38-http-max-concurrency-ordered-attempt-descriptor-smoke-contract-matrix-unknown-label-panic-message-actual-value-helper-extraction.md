# M38-S251 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic-Message Actual-Value Helper Extraction

## What it is

M38-S251 extracts unknown-label panic-message actual-value decoding into a dedicated helper.

## Why it exists

Panic payload message decoding was inline in the unknown-label panic-message assertion path, so actual-message extraction was not isolated for reuse.

## How it works

- Added actual-message helper:
  - `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_actual_panic_message(...)`
- Migrated unknown-label panic-message assertion flow to decode panic payload messages through the helper.
- Kept panic payload decoding behavior deterministic and unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
