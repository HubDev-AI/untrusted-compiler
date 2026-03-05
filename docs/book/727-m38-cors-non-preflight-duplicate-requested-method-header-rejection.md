# M38-S104 CORS Non-Preflight Duplicate Requested-Method Header Rejection

## What it is

M38-S104 hardens runtime CORS handling by rejecting non-preflight requests that carry duplicate `Access-Control-Request-Method` header lines.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Method` is a preflight-only negotiation header. In non-preflight requests, duplicate occurrences are malformed protocol shape and should fail with explicit diagnostics instead of collapsing into generic handling.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime counts `Access-Control-Request-Method` header-line occurrences.
2. If count is greater than one:
   - response is `400 Bad Request`,
   - body is `cors request duplicate requested method header`,
   - CORS allow-origin header is omitted.
3. Single-occurrence non-preflight misuse still follows the existing deterministic rejection branch.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_method_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one extra header-count branch in non-preflight CORS handling.
- Error diagnostics become more explicit for duplicate-header malformed requests while preserving prior rejection semantics.

## Next

1. Continue tightening malformed CORS request-shape rejection in incremental runtime slices.
2. Keep runtime hardening synchronized with deterministic e2e tests and roadmap/book documentation.
