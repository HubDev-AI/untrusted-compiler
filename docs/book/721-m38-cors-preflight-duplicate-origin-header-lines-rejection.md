# M38-S98 CORS Preflight Duplicate Origin Header-Line Rejection

## What it is

M38-S98 hardens runtime CORS preflight handling by rejecting duplicate `Origin` header lines as malformed input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, duplicate `Origin` header lines were not explicitly rejected, and preflight handling could proceed using a single extracted value. That introduces ambiguity for an origin-critical security boundary.

## How it works

1. Runtime preflight branch now counts `Origin` header occurrences in raw request headers.
2. If `Origin` appears more than once:
   - response is `400 Bad Request`,
   - body is `cors preflight duplicate origin header`.
3. Existing single-origin branches remain unchanged:
   - missing origin -> `400`,
   - invalid origin -> `400`,
   - valid origin continues to requested-method/requested-headers checks.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_origin_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_origin_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_origin_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Preflight parsing does one additional header-occurrence check for `Origin`.
- Runtime behavior is stricter and more deterministic for malformed origin-header input.

## Next

1. Continue CORS runtime hardening with explicit malformed-input branches and deterministic diagnostics.
2. Keep each slice scoped to runtime branch + e2e proof + roadmap/book updates.
