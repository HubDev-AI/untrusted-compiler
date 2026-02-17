# M38-S162 HTTP Max-Concurrency Connector-Thread Helper Normalization

## What it is

M38-S162 normalizes connector thread spawn/join handling for queue/security contention paths.

## Why it exists

To remove repeated connector-thread blocks that could drift in failure handling.

## How it works

- Queue and security contention tests now use a shared connector-thread helper path.
- First/second connector failures still emit case-specific diagnostics.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
