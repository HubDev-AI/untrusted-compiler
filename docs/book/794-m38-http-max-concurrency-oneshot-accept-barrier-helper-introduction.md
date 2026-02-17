# M38-S167 HTTP Max-Concurrency Oneshot Accept-Barrier Helper Introduction

## What it is

M38-S167 introduces a helper for oneshot accept-barrier synchronization.

## Why it exists

To remove repeated literal sleep calls and keep late/burst timing intent explicit.

## How it works

- `wait_for_oneshot_accept_barrier(...)` wraps the fixed accept-barrier delay.
- Late and burst contention paths now call the helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_`
