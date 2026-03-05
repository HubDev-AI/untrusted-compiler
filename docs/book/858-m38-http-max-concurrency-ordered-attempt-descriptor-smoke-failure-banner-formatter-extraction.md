# M38-S231 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Failure-Banner Formatter Extraction

## What it is

M38-S231 extracts ordered descriptor smoke failure-banner rendering into dedicated formatter helpers.

## Why it exists

After S230 extracted success-path banner formatting, wrapped smoke failure rendering still composed message text inline inside the catch-unwind path.

## How it works

- Added failure-banner formatter helper:
  - `ordered_contention_smoke_failure_banner(...)`
- Added payload-to-failure-banner helper:
  - `ordered_contention_smoke_failure_banner_from_payload(...)`
- Updated wrapped smoke failure path:
  - `run_ordered_contention_smoke_expectation_with_failure_context(...)`
  to panic using formatter-helper output.
- Added deterministic contract coverage:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
