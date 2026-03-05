# M38-S95 CORS Preflight Duplicate Requested-Headers Rejection

## What it is

M38-S95 hardens runtime CORS preflight handling by rejecting duplicate tokens in `Access-Control-Request-Headers` as malformed input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After M38-S94, malformed/empty requested-header inputs were rejected, but duplicate tokens could still pass through as if they were a normal request. Duplicate header tokens add avoidable ambiguity and can mask malformed client behavior.

## How it works

1. Requested-headers validator now tracks seen header tokens case-insensitively.
2. Duplicate token detection marks preflight input as invalid.
3. Duplicate requested-headers preflight now returns:
   - `400 Bad Request`, body `cors preflight requested headers invalid`.
4. Existing branches remain unchanged:
   - malformed token -> `400`,
   - disallowed token -> `403`,
   - allowed distinct tokens -> `204`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_tokens`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_empty_requested_headers_value`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Validator logic is slightly more complex due to duplicate-tracking state.
- Runtime behavior is stricter but more deterministic for malformed preflight header lists.

## Next

1. Continue CORS/runtime hardening with explicit deterministic malformed-input diagnostics.
2. Keep each slice isolated with direct e2e coverage and synced roadmap/book updates.
