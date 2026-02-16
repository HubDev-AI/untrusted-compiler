# M38-S105 CORS Non-Preflight Duplicate Requested-Headers Header Rejection

## What it is

M38-S105 hardens runtime CORS handling by rejecting non-preflight requests that carry duplicate `Access-Control-Request-Headers` header lines.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Headers` is a preflight-only negotiation header. Duplicate header lines on non-preflight traffic are malformed request shape and should be rejected with deterministic diagnostics.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime counts `Access-Control-Request-Headers` header-line occurrences.
2. If count is greater than one:
   - response is `400 Bad Request`,
   - body is `cors request duplicate requested headers header`,
   - CORS allow-origin header is omitted.
3. Single-occurrence non-preflight misuse continues through the existing deterministic rejection branch.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_headers_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one extra header-count branch in non-preflight CORS handling.
- Duplicate malformed-request diagnostics become explicit while preserving prior rejection semantics.

## Next

1. Continue incremental malformed CORS request-shape hardening with deterministic diagnostics.
2. Keep runtime, e2e tests, roadmap, and book chapters aligned per slice.
