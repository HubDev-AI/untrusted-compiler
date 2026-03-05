# M38-S172 HTTP Max-Concurrency Staged-Connection Helper Foundation

## What it is

M38-S172 adds a shared staged-connection setup foundation for late/burst contention tests.

## Why it exists

To centralize staged connection retry/timeout wiring and avoid setup drift.

## How it works

- Added `connect_staged_stream_or_terminate(...)`.
- Helper wraps connect retry + child terminate fallback + read-timeout setup.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
