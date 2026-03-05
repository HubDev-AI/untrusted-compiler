# M38-S96 CORS Preflight Duplicate Requested-Method Header Rejection

## What it is

M38-S96 hardens runtime CORS preflight handling by rejecting duplicate `Access-Control-Request-Method` header lines as malformed input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, duplicate requested-method headers were ambiguous and runtime would effectively use the first parsed value. That could hide malformed client behavior and make CORS preflight outcomes less deterministic.

## How it works

1. Runtime now counts occurrences of specific request headers in the raw request header block.
2. CORS preflight path checks `Access-Control-Request-Method` occurrence count before value validation.
3. If the header appears more than once:
   - response is `400 Bad Request`,
   - body is `cors preflight duplicate requested method header`.
4. Existing single-header branches remain unchanged:
   - missing -> `400`,
   - invalid token -> `400`,
   - disallowed method -> `403`,
   - allowed -> `204`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_method_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_token_is_invalid`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime request parsing adds an explicit duplicate-header scan for one preflight critical header.
- Deterministic malformed-input handling improves observability and avoids first-value ambiguity.

## Next

1. Continue tightening CORS middleware parsing edges with explicit deterministic reject paths.
2. Keep runtime, e2e coverage, roadmap, and book updates synchronized per slice.
