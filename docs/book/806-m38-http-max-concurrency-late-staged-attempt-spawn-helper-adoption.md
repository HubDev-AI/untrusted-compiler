# M38-S179 HTTP Max-Concurrency Late Staged-Attempt Spawn Helper Adoption

## What it is

M38-S179 migrates late contention attempt bootstrap to the staged-attempt spawn helper.

## Why it exists

To remove duplicated late-path bootstrap boilerplate.

## How it works

- Late default and low-timeout tests now call `spawn_staged_contention_attempt(...)`.
- Late runtime contention contracts are unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_`
