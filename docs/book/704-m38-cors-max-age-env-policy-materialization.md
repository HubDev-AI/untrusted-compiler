# M38-S81 CORS Max-Age Env Policy Materialization

## What it is

M38-S81 adds real runtime policy materialization for CORS preflight max-age and deterministic fallback handling for malformed values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS policy includes `max_age_seconds`, but runtime preflight handling was fixed at `600` regardless of env policy. This slice makes max-age configurable through env policy while preserving safe defaults under invalid inputs.

## How it works

1. Extended runtime CORS policy state with `max_age_seconds`.
2. During env policy loading:
   - parse `SEC4_RT_CORS_MAX_AGE_SECONDS`
   - accept only positive values
   - fallback to `600` for invalid/non-positive values.
3. During `cors.fromPolicy()` materialization:
   - apply loaded max-age value into router preflight config.
4. Added clang-gated e2e tests:
   - `c_bin_http_runtime_applies_cors_max_age_env_value_on_preflight_when_enabled`
   - `c_bin_http_runtime_applies_cors_max_age_invalid_env_falls_back_to_default_on_preflight`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_max_age_env_value_on_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_max_age_invalid_env_falls_back_to_default_on_preflight`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one small policy-state field and two focused e2e tests.
- Increases test/runtime surface slightly, but closes a real policy materialization gap and improves deterministic config behavior.

## Next

1. Continue runtime middleware policy materialization hardening for remaining env-backed fields (auth/csrf/security headers) where policy values should be enforced deterministically.
2. Keep shipping each runtime change with e2e proof and roadmap/book traceability.
