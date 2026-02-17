# M38-S156 HTTP Max-Concurrency Response Contract Predicate Helper Introduction

## What it is

M38-S156 introduces canonical response-contract predicates for contention assertions.

## Why it exists

To prevent contract drift between queue, late, burst, and security parity tests.

## How it works

- Shared predicates now validate success/throttle envelopes.
- Trace-specific variants enforce ordered contention paths (`rt-1`, `rt-2`, `rt-3`).
- Security-header parity checks use one canonical header predicate.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
