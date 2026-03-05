# M38-S109 CORS Preflight Private-Network Header Value Validation

## What it is

M38-S109 hardens runtime CORS preflight handling by validating `Access-Control-Request-Private-Network` value shape when the header is present.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Private-Network` is part of preflight negotiation for Private Network Access. Accepting arbitrary values weakens deterministic protocol handling and makes malformed requests look valid.

## How it works

1. In CORS preflight branch (`method == OPTIONS`), runtime checks `Access-Control-Request-Private-Network` if present.
2. Allowed value is case-insensitive `true` only.
3. Any other value:
   - response is `400 Bad Request`,
   - body is `cors preflight private-network header invalid`,
   - preflight allow-methods header block is omitted.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_authorization_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime introduces one value-validation branch for optional preflight private-network header.
- Non-conforming private-network preflight values now fail fast with deterministic diagnostics.

## Next

1. Continue CORS malformed-input hardening with deterministic runtime outcomes.
2. Keep runtime behavior, e2e tests, roadmap, and chapter docs synchronized per slice.
