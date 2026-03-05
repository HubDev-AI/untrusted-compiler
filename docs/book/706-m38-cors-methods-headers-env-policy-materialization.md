# M38-S83 CORS Methods/Headers Env Policy Materialization

## What it is

M38-S83 implements real runtime materialization for CORS allowed-methods and allowed-headers env policy lists, including deterministic fallback on invalid input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Preflight CORS responses previously used fixed method/header lists. That ignored env policy intent and made runtime behavior less configurable than the policy surface. This slice closes that gap and hardens malformed list handling.

## How it works

1. Extended runtime CORS policy state with:
   - `allow_methods`
   - `allow_headers`
2. Added env loading for:
   - `SEC4_RT_CORS_ALLOWED_METHODS`
   - `SEC4_RT_CORS_ALLOWED_HEADERS`
3. Added deterministic validators:
   - methods list must contain known verbs (`GET/POST/PUT/PATCH/DELETE/OPTIONS`)
   - header list must contain valid header-name tokens
4. On invalid lists, runtime clamps to defaults:
   - methods: `GET, POST, PUT, PATCH, DELETE, OPTIONS`
   - headers: `content-type, authorization`
5. `withCors(..., cors.fromPolicy())` now applies validated values to preflight responses.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_methods_and_headers_from_env_on_preflight`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_methods_and_headers_invalid_env_fall_back_to_defaults`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds validation logic and policy-state fields in runtime CORS loader.
- Slightly increases e2e test/runtime complexity, but aligns runtime behavior with policy and prevents malformed list drift.

## Next

1. Continue middleware policy materialization hardening for remaining env-backed fields where runtime still relies on fixed defaults.
2. Keep each runtime policy change paired with deterministic e2e coverage and roadmap/book traceability.
