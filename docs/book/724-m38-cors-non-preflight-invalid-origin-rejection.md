# M38-S101 CORS Non-Preflight Invalid-Origin Rejection

## What it is

M38-S101 hardens runtime CORS handling by rejecting non-preflight requests that carry invalid `Origin` values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S98 and M38-S99 tightened duplicate-origin handling, but non-preflight requests with a single malformed origin token could still flow through and receive fallback CORS behavior. This slice makes malformed origin values fail fast for non-preflight requests.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime reads request `Origin`.
2. If `Origin` is present and invalid (`*` or non-origin token shape):
   - response is `400 Bad Request`,
   - body is `cors request origin invalid`,
   - CORS allow-origin header is not emitted.
3. Existing behavior remains:
   - valid-origin requests continue as before,
   - missing-origin requests continue as before,
   - duplicate-origin requests remain covered by M38-S99.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_invalid_origin_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_origin_headers`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict malformed-origin branch for non-preflight CORS requests.
- Behavior is stricter for malformed client origin headers, but deterministic and explicit.

## Next

1. Continue CORS runtime hardening by separating malformed-input and policy-denied branches.
2. Keep incremental runtime/e2e/docs slices for reviewable security progression.
