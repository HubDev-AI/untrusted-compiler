# M38-S92 CORS Preflight Requested-Method Token-Shape Validation

## What it is

M38-S92 hardens runtime CORS preflight handling by validating `Access-Control-Request-Method` token shape before method allowlist checks.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, malformed requested-method values were folded into generic `403 method not allowed`. That made diagnostics less precise and mixed malformed preflight requests with policy-denied but well-formed methods.

## How it works

1. Runtime preflight path now validates `Access-Control-Request-Method` as a method-token:
   - invalid token characters/shape -> `400 Bad Request`, body `cors preflight requested method invalid`.
2. Existing missing-header guard remains:
   - missing header -> `400 Bad Request`, body `cors preflight missing requested method`.
3. Existing allowlist membership guard remains:
   - well-formed but disallowed method -> `403 Forbidden`, body `cors preflight method not allowed`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_token_is_invalid`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Preflight parsing becomes stricter and may reject malformed client traffic that previously mapped to generic policy-denied responses.
- Additional branch-specific diagnostics improve operator debugging and CI determinism.

## Next

1. Continue CORS preflight hardening with similarly explicit malformed-input vs policy-denied diagnostics.
2. Keep each runtime hardening slice minimal: one branch change, one e2e proof, one roadmap/book update.
