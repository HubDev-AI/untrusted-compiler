# M38-S146 HTTP Max-Concurrency Late-Connection Spawn-Helper Adoption

## What it is

M38-S146 migrates late-connection contention tests (default and low-timeout) to shared spawn helper wiring.

## Why it exists

Late-connection variants repeated identical spawn env setup.

## How it works

- Both late-connection tests now use `spawn_max_concurrency_oneshot_binary(...)`.
- Existing deterministic status/trace/body assertions remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Apply helper across burst-ingress paths (`M38-S147`).
