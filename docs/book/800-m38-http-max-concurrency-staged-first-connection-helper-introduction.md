# M38-S173 HTTP Max-Concurrency Staged First-Connection Helper Introduction

## What it is

M38-S173 introduces a dedicated helper for first staged connection setup.

## Why it exists

To keep first-connection retry budget and diagnostics deterministic across late/burst paths.

## How it works

- Added `connect_staged_first_stream_or_terminate(...)`.
- Helper fixes first staged connection retry budget at 800 attempts.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_`
