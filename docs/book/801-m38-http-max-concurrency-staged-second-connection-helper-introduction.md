# M38-S174 HTTP Max-Concurrency Staged Second-Connection Helper Introduction

## What it is

M38-S174 introduces a dedicated helper for second staged connection setup.

## Why it exists

To centralize second-connection retry behavior and failure envelopes.

## How it works

- Added `connect_staged_second_stream_or_terminate(...)`.
- Helper fixes second staged connection retry budget at 400 attempts.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_`
