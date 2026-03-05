# M38-S226 HTTP Max-Concurrency Ordered-Attempt Descriptor Smoke Iterator Failure-Context Enrichment

## What it is

M38-S226 enriches table-driven ordered descriptor smoke harness failures with deterministic per-case context.

## Why it exists

As ordered smoke coverage moved to compact iterator traversal, any single failure needed explicit case metadata to keep debugging speed high.

## How it works

- Added panic payload normalization helper:
  - `panic_payload_message(...)`
- Added wrapped smoke execution helper:
  - `run_ordered_contention_smoke_expectation_with_failure_context(...)`
- Updated smoke harness traversal to run each expectation through the context wrapper, enriching failures with:
  - case label
  - module name
  - fixture name

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`
- `scripts/test-roadmap-closure-gate-alignment.sh`
