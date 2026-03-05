# M38-S94 CORS Preflight Empty Requested-Headers Rejection

## What it is

M38-S94 hardens runtime CORS preflight handling by rejecting explicitly empty `Access-Control-Request-Headers` values as malformed input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After M38-S93, malformed requested-header tokens were classified as `400`, but an explicitly present empty header value could still bypass malformed-input classification. That kept one malformed branch under-specified.

## How it works

1. Runtime requested-headers validator now distinguishes:
   - missing header (no validation path),
   - empty provided value (invalid),
   - malformed token,
   - disallowed token.
2. Empty requested-headers value now maps to:
   - `400 Bad Request`, body `cors preflight requested headers invalid`.
3. Existing branches remain:
   - malformed token -> `400`,
   - well-formed but disallowed -> `403`,
   - allowed -> `204`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_empty_requested_headers_value`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Preflight input handling is stricter and rejects more malformed traffic explicitly.
- Deterministic malformed-input diagnostics improve debugging and policy/audit clarity.

## Next

1. Continue runtime CORS hardening by isolating malformed protocol input from policy outcomes.
2. Preserve the same slice structure: runtime branch, direct e2e proof, roadmap/book update.
