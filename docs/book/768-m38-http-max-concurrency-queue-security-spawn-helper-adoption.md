# M38-S145 HTTP Max-Concurrency Queue/Security Spawn-Helper Adoption

## What it is

M38-S145 migrates queue-boundary and security-header parity contention tests to shared spawn helper wiring.

## Why it exists

Queue/security tests had duplicate spawn setup and should use one deterministic helper path.

## How it works

- Queue-boundary tests now spawn through `spawn_max_concurrency_oneshot_binary(...)`.
- Security-header parity test uses the same helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Apply helper across late-connection paths (`M38-S146`).
