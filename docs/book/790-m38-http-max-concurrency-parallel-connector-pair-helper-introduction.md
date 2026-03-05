# M38-S163 HTTP Max-Concurrency Parallel Connector-Pair Helper Introduction

## What it is

M38-S163 introduces a canonical helper that establishes two connector attempts in parallel.

## Why it exists

To keep retry cadence and connector-failure fallback behavior consistent.

## How it works

- `connect_pair_in_parallel_or_terminate(...)` wraps dual connector threads.
- On failed connect, helper terminates and waits on the runtime child before failing the test.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
