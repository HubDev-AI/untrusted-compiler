# M38-S77 CORS Enabled Invalid Env Fallback

## What it is

M38-S77 adds runtime e2e coverage proving invalid `SEC4_RT_CORS_ENABLED` env values deterministically keep default CORS-enabled behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The top-level CORS enable toggle is env-driven and parsed with strict boolean handling where malformed values must fall back to the safe runtime default (`enabled=true`). This contract needed explicit e2e coverage so invalid env tokens cannot silently disable CORS behavior.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_cors_enabled_invalid_env_falls_back_to_enabled_by_default`
2. The harness runs a c-bin HTTP service with:
   - `SEC4_RT_CORS_ENABLED=MAYBE` (invalid boolean token)
   - request origin `https://app.example.com`
3. Test asserts deterministic fallback behavior:
   - response remains `HTTP/1.1 200 OK`
   - response still emits `Access-Control-Allow-Origin: *`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_enabled_invalid_env_falls_back_to_enabled_by_default`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and slight runtime-suite overhead.
- Improves confidence that malformed env toggles cannot disable baseline CORS behavior unexpectedly.

## Next

1. Continue CORS env-hardening coverage for remaining toggles and numeric fields so all policy-driven CORS controls are locked by deterministic e2e contracts.
2. Keep runtime fallback behavior and roadmap/book contracts aligned slice-by-slice.
