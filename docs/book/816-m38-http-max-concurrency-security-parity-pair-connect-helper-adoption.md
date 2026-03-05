# M38-S189 HTTP Max-Concurrency Security-Parity Pair-Connect Helper Adoption

## What it is

M38-S189 migrates security parity pair connect/timeout setup to the pair-connect helper.

## Why it exists

To remove duplicated pair connect + timeout setup in the security parity branch.

## How it works

- Security parity branch now calls `connect_pair_streams_or_terminate(...)`.
- Security parity contention contract remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
