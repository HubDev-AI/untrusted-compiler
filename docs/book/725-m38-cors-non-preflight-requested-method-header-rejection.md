# M38-S102 CORS Non-Preflight Requested-Method Header Rejection

## What it is

M38-S102 hardens runtime CORS handling by rejecting non-preflight requests that include `Access-Control-Request-Method`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Method` is a preflight-only header. Accepting it on non-preflight requests weakens protocol-shape enforcement and allows malformed request shapes to reach normal dispatch paths.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime checks whether `Access-Control-Request-Method` is present.
2. If present:
   - response is `400 Bad Request`,
   - body is `cors request requested method header not allowed`,
   - CORS allow-origin header is omitted.
3. Existing non-preflight behavior remains unchanged for valid request shapes.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_invalid_origin_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime introduces a stricter protocol-shape branch for CORS-enabled non-preflight requests.
- Non-conforming clients sending preflight-only headers on normal requests now fail fast with deterministic diagnostics.

## Next

1. Continue CORS runtime strictness by rejecting other preflight-only header misuse on non-preflight requests.
2. Keep each hardening slice bounded with direct e2e proof and synced roadmap/book updates.
