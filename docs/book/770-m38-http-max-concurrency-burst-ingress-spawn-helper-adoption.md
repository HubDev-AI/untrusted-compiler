# M38-S147 HTTP Max-Concurrency Burst-Ingress Spawn-Helper Adoption

## What it is

M38-S147 migrates burst-ingress contention tests (default and low-timeout) to shared spawn helper wiring.

## Why it exists

Burst variants repeated identical spawn setup and should share one canonical path.

## How it works

- Both burst-ingress tests now call `spawn_max_concurrency_oneshot_binary(...)`.
- Deterministic trace/order + throttle body assertions remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Introduce shared fixture-build helper (`M38-S148`).
