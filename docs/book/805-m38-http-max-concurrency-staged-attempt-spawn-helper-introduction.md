# M38-S178 HTTP Max-Concurrency Staged-Attempt Spawn Helper Introduction

## What it is

M38-S178 introduces a helper for late/burst attempt bootstrap.

## Why it exists

To centralize `attempt start + free port + oneshot spawn` bootstrap wiring.

## How it works

- Added `spawn_staged_contention_attempt(...)`.
- Helper returns `(attempt_started, port, child)` for each contention attempt.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_`
