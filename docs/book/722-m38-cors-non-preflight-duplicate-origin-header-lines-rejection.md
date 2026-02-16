# M38-S99 CORS Non-Preflight Duplicate Origin Header-Line Rejection

## What it is

M38-S99 hardens runtime CORS handling by rejecting duplicate `Origin` header lines on non-preflight requests.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S98 covered duplicate origin lines for preflight requests. Non-preflight requests still accepted duplicate origin lines and could proceed with ambiguous origin context. This slice closes that gap.

## How it works

1. Runtime counts `Origin` header occurrences once request headers are parsed.
2. If CORS is enabled and request is non-preflight (`method != OPTIONS`) with duplicate origin lines:
   - response is `400 Bad Request`,
   - body is `cors request duplicate origin header`,
   - response omits CORS allow-origin header emission.
3. Existing behavior remains:
   - normal single-origin/non-origin requests continue through router/auth/csrf/security paths,
   - preflight duplicate-origin requests remain handled by M38-S98 branch.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_origin_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_origin_headers`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict malformed-input branch for CORS-enabled non-preflight traffic.
- Requests with duplicated origin lines now fail fast instead of being tolerated with ambiguous origin interpretation.

## Next

1. Continue narrowing malformed CORS input acceptance with explicit deterministic diagnostics.
2. Keep each hardening increment vertically complete: runtime change, e2e, roadmap, and book chapter.
