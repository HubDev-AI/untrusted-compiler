# M38-S93 CORS Preflight Requested-Headers Token-Shape Validation

## What it is

M38-S93 hardens runtime CORS preflight handling by separating malformed `Access-Control-Request-Headers` values from policy-denied header names.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, malformed requested-header tokens and disallowed (but well-formed) requested headers shared the same `403 headers not allowed` response. That collapsed malformed-input diagnostics into policy-denied diagnostics and reduced operator clarity.

## How it works

1. Runtime requested-headers validator now tracks invalid-token shape errors.
2. Preflight branch maps validator outcomes deterministically:
   - malformed requested-headers token -> `400 Bad Request`, body `cors preflight requested headers invalid`.
   - well-formed but not in allowlist -> `403 Forbidden`, body `cors preflight headers not allowed`.
3. Allowed requested-headers behavior remains unchanged (`204 No Content` with CORS preflight headers).

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Stricter malformed-input classification introduces one extra runtime branch in preflight handling.
- Diagnostics become more explicit and deterministic for CORS debugging and CI assertions.

## Next

1. Continue preflight/runtime hardening with explicit malformed vs policy-denied branches.
2. Keep runtime, e2e coverage, roadmap entry, and book chapter updates coupled in each slice.
