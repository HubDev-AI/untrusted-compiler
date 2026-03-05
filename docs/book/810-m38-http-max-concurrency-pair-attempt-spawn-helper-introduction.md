# M38-S183 HTTP Max-Concurrency Pair-Attempt Spawn Helper Introduction

## What it is

M38-S183 introduces a dedicated helper for queue/security pair-attempt spawning.

## Why it exists

To centralize attempt start, port selection, and oneshot spawn bootstrapping.

## How it works

- Added `spawn_pair_contention_attempt(...)`.
- Helper reuses staged contention spawn behavior and keeps spawn diagnostics stable.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
