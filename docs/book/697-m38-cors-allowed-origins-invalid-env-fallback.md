# M38-S74 CORS Allowed-Origins Invalid Env Fallback

## What it is

M38-S74 adds runtime CORS hardening and e2e coverage proving invalid `SEC4_RT_CORS_ALLOWED_ORIGINS` env values deterministically fall back to wildcard-origin baseline.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS allow-origin token comes from env and is emitted into HTTP headers. Without header-value validation, malformed env values (for example containing CR/LF) could be reflected into header output.

This slice hardens runtime loading and locks fallback behavior to `*`.

## How it works

1. Updated runtime CORS policy loading in `sec4_rt_load_cors_policy_from_env(...)`:
   - parse first token from `SEC4_RT_CORS_ALLOWED_ORIGINS`
   - validate token via header-value validator
   - fallback to `*` when token is missing/invalid.
2. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
3. Harness starts a c-bin HTTP service with invalid `SEC4_RT_CORS_ALLOWED_ORIGINS` (newline payload) and asserts:
   - `Access-Control-Allow-Origin: *`
   - invalid origin value is not reflected in response.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one runtime validation check and one focused e2e test.
- Slightly increases CORS-policy loading logic, but prevents malformed env inputs from reaching header output.

## Next

1. Add CORS allow-credentials invalid-env fallback coverage (`M38-S75`) to lock default credentials-disabled behavior on malformed boolean tokens.
2. Continue env/policy hardening across CORS toggles with explicit e2e contracts.
