# M38-S153 HTTP Max-Concurrency Connector Retry Helper Introduction

## What it is

M38-S153 introduces canonical connector retry helpers for contention tests.

## Why it exists

To remove duplicated retry loops and keep connection-attempt cadence deterministic.

## How it works

- `connect_with_retry(...)` centralizes retry attempts and sleep cadence.
- `connect_with_retry_or_terminate(...)` centralizes child terminate+wait failure fallback.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
