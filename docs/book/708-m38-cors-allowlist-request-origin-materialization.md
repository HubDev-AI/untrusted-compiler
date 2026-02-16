# M38-S85 CORS Allowlist Request-Origin Materialization

## What it is

M38-S85 hardens runtime CORS allow-origin behavior so `SEC4_RT_CORS_ALLOWED_ORIGINS` can be a real allowlist (CSV), not just a single static token.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, runtime policy loading copied only the first token from `SEC4_RT_CORS_ALLOWED_ORIGINS`, so multi-origin allowlists were effectively ignored. That prevented deterministic allowlist behavior for apps serving more than one trusted frontend origin.

## How it works

1. Added full CSV validation for CORS origins:
   - accepts `*` or header-safe origin tokens,
   - rejects malformed/empty tokens,
   - invalid env values fall back to wildcard (`*`).
2. Kept the configured allowlist string in router CORS state.
3. Updated CORS success/preflight header assembly:
   - if allow-origin is wildcard, emits `*`,
   - if allowlist is non-wildcard and request `Origin` matches a configured token, reflects the request origin,
   - if request origin does not match, deterministically falls back to the first configured token.
4. Moved per-request CORS header assembly to run after request headers are parsed, so origin matching can use the request `Origin` value.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_allowed_origins_allowlist_non_matching_origin_falls_back_to_first_token_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime now performs request-aware allow-origin resolution, which adds a small amount of logic in CORS header assembly.
- Parse-failure (`400 bad request`) responses remain deterministic but no longer precompute request-aware CORS headers before request parsing.

## Next

1. Continue materializing remaining env-backed middleware policy fields with the same pattern (strict validation, deterministic fallback, e2e coverage).
2. Keep CORS/runtime policy behavior aligned between preflight and success/error response paths.
