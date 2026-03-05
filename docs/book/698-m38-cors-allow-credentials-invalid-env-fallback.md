# M38-S75 CORS Allow-Credentials Invalid Env Fallback

## What it is

M38-S75 adds runtime e2e coverage proving invalid `SEC4_RT_CORS_ALLOW_CREDENTIALS` env values deterministically keep default credentials-disabled behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS allow-credentials is controlled by strict env-boolean parsing with default `false`. This fallback behavior needed explicit e2e contract coverage so malformed env tokens cannot silently enable credentialed CORS responses.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_cors_allow_credentials_invalid_env_falls_back_to_disabled_when_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_CORS_ALLOW_CREDENTIALS=MAYBE` (invalid)
3. Test asserts deterministic fallback behavior:
   - response includes `Access-Control-Allow-Origin: *`
   - response does not include `Access-Control-Allow-Credentials: true`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allow_credentials_invalid_env_falls_back_to_disabled_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and slight runtime-suite overhead.
- Improves confidence that malformed env inputs cannot broaden CORS credential exposure.

## Next

1. Add CORS require-vary-origin invalid-env fallback coverage (`M38-S76`) to lock default `Vary: Origin` disabled behavior under malformed env values.
2. Continue CORS env-hardening slices until all active CORS toggles have explicit runtime contract tests.
