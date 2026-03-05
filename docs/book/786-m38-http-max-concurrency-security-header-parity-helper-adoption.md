# M38-S159 HTTP Max-Concurrency Security-Header Parity Helper Adoption

## What it is

M38-S159 migrates max-concurrency throttle security-header parity coverage to helper-driven assertions.

## Why it exists

To keep parity checks and contention contracts aligned under one deterministic assertion surface.

## How it works

- Security parity path now uses canonical connect/I/O/wait helpers.
- Success/throttle selection and envelope checks use shared predicates.
- Header parity check uses canonical `response_has_default_security_headers(...)`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
