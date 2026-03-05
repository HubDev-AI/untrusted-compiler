# M38-S112 CORS Preflight Private-Network Default-Deny

## What it is

M38-S112 hardens runtime CORS preflight handling by default-denying `Access-Control-Request-Private-Network: true` requests.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Private Network Access requires explicit opt-in. Accepting private-network preflight requests by default creates implicit trust expansion for internal network targets.

## How it works

1. In CORS preflight branch (`method == OPTIONS`), runtime checks `Access-Control-Request-Private-Network`.
2. If header is present and valid (`true`), runtime returns deterministic deny by default:
   - `403 Forbidden`,
   - body `cors preflight private-network not allowed`,
   - no preflight allow-methods header block.
3. Existing malformed-input checks remain active:
   - duplicate private-network header -> `400`,
   - invalid value -> `400`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_private_network_header_when_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Default behavior is stricter and may block clients that expected implicit private-network preflight allow.
- Security posture improves by requiring explicit future policy wiring before private-network preflight is accepted.

## Next

1. Add explicit policy/env opt-in path for private-network preflight allow in a dedicated follow-up slice.
2. Keep runtime/e2e/docs alignment deterministic per hardening slice.
