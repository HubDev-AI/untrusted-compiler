# M38-S230 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Banner Formatter-Helper Contract Extraction

## What it is

M38-S230 extracts ordered descriptor smoke banner rendering into a dedicated formatter helper with explicit contract coverage.

## Why it exists

After S229 prefix constant extraction, banner rendering still lived in a wrapper function, making formatter behavior less explicit and harder to validate independently.

## How it works

- Added dedicated formatter helper:
  - `ordered_contention_smoke_banner_formatter(...)`
- Migrated wrapper renderer:
  - `ordered_contention_smoke_case_banner(...)`
  to delegate to the formatter helper.
- Extended banner-contract coverage to assert:
  - exact formatter output shape
  - parity between wrapper-rendered and formatter-rendered banner output.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
