# M38-S227 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Case-Banner Tracing

## What it is

M38-S227 adds deterministic per-case banner tracing to ordered descriptor smoke harness execution.

## Why it exists

After S226 failure-context enrichment, compact smoke harness debugging still benefits from explicit run-time case banners to show execution progression across table-driven fixtures.

## How it works

- Added deterministic banner helper:
  - `ordered_contention_smoke_case_banner(...)`
- Updated smoke harness execution wrapper to emit per-case banners before fixture run.
- Reused the same case-banner string in wrapped panic messages for consistent trace/failure correlation.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
