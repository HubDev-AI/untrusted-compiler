# M38-S110 CORS Preflight Duplicate Private-Network Header Rejection

## What it is

M38-S110 hardens runtime CORS preflight handling by rejecting duplicate `Access-Control-Request-Private-Network` header lines.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Private-network preflight negotiation should have deterministic, single-header request shape. Duplicate header lines create ambiguous negotiation inputs and should be rejected as malformed traffic.

## How it works

1. In CORS preflight branch (`method == OPTIONS`), runtime counts `Access-Control-Request-Private-Network` header-line occurrences.
2. If count is greater than one:
   - response is `400 Bad Request`,
   - body is `cors preflight duplicate private-network header`,
   - preflight allow-methods header block is omitted.
3. Single-header preflight requests continue through existing value validation (`true` only) and normal preflight checks.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_private_network_headers`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one additional header-count branch in preflight CORS handling.
- Duplicate private-network malformed preflight requests now fail fast with deterministic diagnostics.

## Next

1. Continue deterministic malformed CORS request-shape hardening in incremental slices.
2. Keep runtime behavior, e2e tests, roadmap, and chapter docs synchronized.
