# M38-S248 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic Message Assertion Helper Extraction

## What it is

M38-S248 extracts unknown-label panic message equality checks into a dedicated assertion helper.

## Why it exists

Panic message decoding and expected-message equality checks were inline in the panic assertion flow, which made message-contract behavior harder to audit separately.

## How it works

- Added message assertion helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message(...)`
- Routed unknown-label panic assertion flow through the helper for deterministic message checks.
- Preserved exact panic message contract equality against the canonical panic-message renderer.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
