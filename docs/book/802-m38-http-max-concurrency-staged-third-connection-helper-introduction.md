# M38-S175 HTTP Max-Concurrency Staged Third-Connection Helper Introduction

## What it is

M38-S175 introduces a dedicated helper for third staged connection setup in burst paths.

## Why it exists

To remove duplicate third-connection retry/timeout setup logic.

## How it works

- Added `connect_staged_third_stream_or_terminate(...)`.
- Helper fixes third staged connection retry budget at 400 attempts.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_`
