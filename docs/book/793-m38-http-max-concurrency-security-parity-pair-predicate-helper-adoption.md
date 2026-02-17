# M38-S166 HTTP Max-Concurrency Security-Parity Pair Predicate Helper Adoption

## What it is

M38-S166 adds a dedicated pair predicate for security-header parity contention checks.

## Why it exists

To keep success/throttle header parity checks centralized and deterministic.

## How it works

- `pair_success_throttle_with_security_header_parity_holds(...)` combines pair envelope and default-security-header assertions.
- Security parity test now uses one predicate path.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
