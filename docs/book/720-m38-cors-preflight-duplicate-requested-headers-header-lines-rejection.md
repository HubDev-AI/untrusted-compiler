# M38-S97 CORS Preflight Duplicate Requested-Headers Header-Line Rejection

## What it is

M38-S97 hardens runtime CORS preflight handling by rejecting duplicate `Access-Control-Request-Headers` header lines as malformed input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, duplicate requested-headers header lines were tolerated and runtime used the first parsed value. That allowed ambiguous preflight input and reduced determinism in malformed-request diagnostics.

## How it works

1. Preflight branch now counts occurrences of `Access-Control-Request-Headers` in raw request headers.
2. If the header appears more than once:
   - response is `400 Bad Request`,
   - body is `cors preflight duplicate requested headers header`.
3. Existing single-line branches remain unchanged:
   - malformed/empty tokens -> `400`,
   - disallowed tokens -> `403`,
   - allowed tokens -> `204`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_header_lines`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_tokens`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime preflight parsing does one extra header-count scan for this header.
- Behavior is stricter but deterministic for malformed duplicate-header-line input.

## Next

1. Continue CORS middleware hardening by isolating malformed protocol input branches with explicit deterministic diagnostics.
2. Keep each slice minimal: runtime branch change, focused e2e proof, roadmap/book update.
