# M38-S211 HTTP Max-Concurrency Late/Burst Assertion-Loop Helper Consolidation Revalidation

## What it is

M38-S211 captures revalidation after late/burst assertion-loop helper adoption.

## Why it exists

To confirm ordered late/burst helper normalization does not regress staged contention contracts.

## How it works

- Added `run_late_contention_attempt_loop(...)` and `run_burst_contention_attempt_loop(...)`.
- Migrated late default/low-timeout and burst default/low-timeout tests to shared ordered helper loops.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
