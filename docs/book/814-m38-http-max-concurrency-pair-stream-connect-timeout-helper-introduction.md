# M38-S187 HTTP Max-Concurrency Pair-Stream Connect/Timeout Helper Introduction

## What it is

M38-S187 introduces a dedicated helper for pair stream connect + timeout setup.

## Why it exists

To centralize pair stream connection retry and timeout wiring in queue/security paths.

## How it works

- Added `connect_pair_streams_or_terminate(...)`.
- Helper wraps pair connection setup and read-timeout configuration for both streams.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
