# M38-S254 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic Payload Contract Helper Extraction

## What it is

M38-S254 introduces a dedicated panic payload contract helper for unknown-label matrix assertions.

## Why it exists

Unknown-label panic contract orchestration invoked panic payload capture directly, so payload-contract intent was not isolated in a contract-named helper.

## How it works

- Added panic payload contract helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_payload_contract(...)`
- Routed unknown-label panic contract orchestration through the payload contract helper.
- Preserved deterministic payload contract behavior.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
