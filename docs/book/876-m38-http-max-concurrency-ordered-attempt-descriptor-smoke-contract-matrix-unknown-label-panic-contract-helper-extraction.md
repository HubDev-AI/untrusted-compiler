# M38-S249 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic Contract Helper Extraction

## What it is

M38-S249 adds a dedicated unknown-label panic contract helper and routes unknown-label contract entry through it.

## Why it exists

Even after payload and message helper extraction, unknown-label contract invocation still bound directly to the panic assertion helper, leaving one remaining orchestration seam inline.

## How it works

- Added panic contract helper:
  - `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_contract(...)`
- Migrated the no-argument unknown-label contract helper to delegate through the panic contract helper with the shared unknown-label constant.
- Kept matrix case-label contract coverage behavior unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
