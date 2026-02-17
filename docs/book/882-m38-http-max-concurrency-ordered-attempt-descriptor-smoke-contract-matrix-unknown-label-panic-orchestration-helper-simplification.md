# M38-S255 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Contract Matrix Unknown-Label Panic Orchestration Helper Simplification

## What it is

M38-S255 simplifies unknown-label panic contract orchestration by removing an intermediate panic helper layer.

## Why it exists

After payload and message contract helper extraction, the intermediate panic helper added indirection without adding behavior.

## How it works

- Removed intermediate unknown-label panic helper from the orchestration path.
- Updated unknown-label panic contract helper to orchestrate payload and message contract helpers directly.
- Kept matrix case-label contract behavior unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
