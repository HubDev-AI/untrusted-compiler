# M38-S76 CORS Require-Vary-Origin Invalid Env Fallback

## What it is

M38-S76 adds runtime e2e coverage proving invalid `SEC4_RT_CORS_REQUIRE_VARY_ORIGIN` env values deterministically keep default vary-origin-disabled behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The `require_vary_origin` CORS toggle is env-driven and parsed via strict boolean logic with default `false`. This fallback behavior needed explicit e2e contract coverage so malformed env tokens cannot silently enable `Vary: Origin` behavior.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_cors_require_vary_origin_invalid_env_falls_back_to_disabled_when_origin_is_non_wildcard`
2. The harness runs a c-bin HTTP service with:
   - `SEC4_RT_CORS_ALLOWED_ORIGINS=https://app.example.com`
   - `SEC4_RT_CORS_REQUIRE_VARY_ORIGIN=MAYBE` (invalid)
3. Test asserts deterministic fallback behavior:
   - response keeps non-wildcard allow-origin
   - response does not emit `Vary: Origin`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_require_vary_origin_invalid_env_falls_back_to_disabled_when_origin_is_non_wildcard`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and slight runtime-suite overhead.
- Improves confidence that malformed env values cannot alter vary-origin behavior unexpectedly.

## Next

1. Add CORS enabled invalid-env fallback coverage (`M38-S77`) to lock default CORS-enabled behavior under malformed toggle values.
2. Continue CORS env-hardening slices until all active CORS toggles have explicit deterministic runtime contracts.
