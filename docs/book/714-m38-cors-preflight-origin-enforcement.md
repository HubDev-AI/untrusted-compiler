# M38-S91 CORS Preflight Origin Enforcement

## What it is

M38-S91 hardens runtime CORS preflight handling by requiring a valid `Origin` header before preflight method/header policy checks.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Preflight requests without a valid origin should not be treated as normal CORS preflight. Without origin presence/shape enforcement, malformed clients could still reach policy evaluation paths and receive less explicit behavior.

## How it works

1. Runtime preflight path now requires `Origin` header:
   - missing header -> `400 Bad Request`, body `cors preflight missing origin`.
2. Runtime validates preflight origin token shape:
   - invalid values (non-origin tokens) -> `400 Bad Request`, body `cors preflight origin invalid`.
3. Only valid-origin preflight requests proceed to:
   - requested-method enforcement,
   - requested-headers enforcement,
   - normal preflight success header emission.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_origin_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_origin_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Preflight handling becomes stricter and may reject malformed traffic that was previously tolerated.
- Diagnostics become explicit and deterministic for missing/invalid origin cases.

## Next

1. Continue runtime boundary hardening for CORS/security middleware with deterministic reject paths.
2. Preserve the same implementation pattern: minimal runtime change, direct e2e proof, roadmap/book update in one slice.
