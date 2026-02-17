# M38-S144 HTTP Max-Concurrency Oneshot Spawn-Helper Dedup

## What it is

M38-S144 adds a canonical helper for spawning oneshot max-concurrency runtime binaries.

## Why it exists

Spawn env setup was duplicated across contention tests and prone to setup drift.

## How it works

- Added `spawn_max_concurrency_oneshot_binary(...)`.
- Helper centralizes standard envs and optional drain-timeout override.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Apply helper across queue/security paths (`M38-S145`).
