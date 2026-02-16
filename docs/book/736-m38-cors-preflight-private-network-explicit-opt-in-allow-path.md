# M38-S113 CORS Preflight Private-Network Explicit Opt-In Allow Path

## What it is

M38-S113 adds an explicit opt-in allow path for private-network CORS preflight requests.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Default-deny behavior is the secure baseline, but controlled deployments may need explicit private-network preflight allowance. The allow path must remain deterministic and policy-driven.

## How it works

1. Runtime loads new CORS policy/env key: `SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK` (default `false`).
2. In preflight handling, `Access-Control-Request-Private-Network: true`:
   - remains denied with `403` when opt-in is disabled,
   - is allowed when opt-in is enabled.
3. On allowed path, preflight response appends:
   - `Access-Control-Allow-Private-Network: true`.
4. Existing malformed-input guards remain active:
   - duplicate private-network header -> `400`,
   - invalid value -> `400`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_private_network_header_when_policy_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_private_network_header_when_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_private_network_headers`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one new policy/env surface area for CORS behavior.
- Maintains secure default-deny while enabling explicit opt-in for private-network preflight use cases.

## Next

1. Continue CORS protocol-shape hardening and strict deterministic diagnostics.
2. Keep runtime behavior, e2e tests, roadmap, and book chapters in sync per slice.
