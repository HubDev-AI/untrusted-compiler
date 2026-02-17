# M38-S228 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Banner-Contract Coverage

## What it is

M38-S228 adds deterministic contract coverage for ordered descriptor smoke case-banner formatting.

## Why it exists

S227 introduced case-banner tracing, so the banner shape now needs explicit regression protection independent of clang/runtime execution.

## How it works

- Added dedicated banner-contract test:
  - `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- Test iterates all ordered descriptor expectations and asserts exact banner rendering:
  - `[ordered-descriptor-smoke] case=... module=... fixture=...`
- Keeps banner formatting synchronized with both tracing output and wrapped failure messages.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
