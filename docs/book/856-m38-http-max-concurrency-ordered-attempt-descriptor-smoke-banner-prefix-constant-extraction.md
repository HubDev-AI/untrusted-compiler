# M38-S229 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Banner-Prefix Constant Extraction

## What it is

M38-S229 extracts the ordered descriptor smoke banner prefix into a shared constant.

## Why it exists

S228 locked banner shape with contract tests, but the banner prefix literal was duplicated between formatter and expected contract strings.

## How it works

- Added canonical prefix constant:
  - `ORDERED_CONTENTION_SMOKE_BANNER_PREFIX`
- Updated banner formatter:
  - `ordered_contention_smoke_case_banner(...)`
  to compose output with the shared prefix constant.
- Updated banner-contract coverage expected strings to use the same shared constant.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
