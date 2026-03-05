# M38-S111 CORS Non-Preflight Duplicate Private-Network Header Rejection

## What it is

M38-S111 hardens runtime CORS handling by rejecting duplicate `Access-Control-Request-Private-Network` header lines on non-preflight requests.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Private-Network` is preflight negotiation metadata. Duplicate header lines on non-preflight traffic are malformed shape and should fail with explicit deterministic diagnostics.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime counts `Access-Control-Request-Private-Network` header-line occurrences.
2. If count is greater than one:
   - response is `400 Bad Request`,
   - body is `cors request duplicate private-network header`,
   - CORS allow-origin header is omitted.
3. Single-header non-preflight misuse keeps existing deterministic rejection branch.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_private_network_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_private_network_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict duplicate-header guard in non-preflight CORS handling.
- Malformed private-network non-preflight traffic now fails fast with deterministic diagnostics.

## Next

1. Continue incremental malformed CORS request-shape hardening slices.
2. Keep runtime behavior, e2e tests, roadmap, and chapter updates in lockstep.
