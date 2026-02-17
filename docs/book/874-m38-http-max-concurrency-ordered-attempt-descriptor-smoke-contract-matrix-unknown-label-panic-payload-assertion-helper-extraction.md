# M38-S247 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic Payload Assertion Helper Extraction

## What it is

M38-S247 extracts unknown-label panic payload capture/assertion into a dedicated helper for smoke contract matrix checks.

## Why it exists

Unknown-label panic payload capture logic was embedded inline in the panic assertion function, making payload contract behavior harder to reuse and evolve.

## How it works

- Added payload helper:
  - `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_payload(...)`
- Moved `catch_unwind` + panic presence assertion into the helper.
- Kept unknown-label panic contract deterministic by returning the captured payload to downstream message assertions.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
