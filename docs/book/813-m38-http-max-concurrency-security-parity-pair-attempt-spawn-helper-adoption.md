# M38-S186 HTTP Max-Concurrency Security-Parity Pair-Attempt Spawn Helper Adoption

## What it is

M38-S186 migrates security parity contention attempt bootstrap to the pair-attempt spawn helper.

## Why it exists

To remove duplicated security parity attempt bootstrap code.

## How it works

- Security parity branch now uses `spawn_pair_contention_attempt(...)`.
- Security parity contention contract remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
