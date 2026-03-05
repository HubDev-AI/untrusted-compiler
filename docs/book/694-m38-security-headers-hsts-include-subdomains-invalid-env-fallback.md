# M38-S71 Security-Headers HSTS IncludeSubDomains Invalid Env Fallback

## What it is

M38-S71 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS` env values deterministically preserve default includeSubDomains-enabled behavior when HSTS is enabled.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The HSTS includeSubDomains toggle is parsed with strict env boolean logic and defaults to enabled. This fallback behavior needed explicit runtime contract coverage to prevent malformed env tokens from silently removing includeSubDomains from emitted HSTS headers.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_hsts_include_subdomains_invalid_env_falls_back_to_enabled_when_hsts_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED=1`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS=31536000`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS=MAYBE` (invalid)
   - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD=0`
3. Test asserts deterministic fallback behavior:
   - response includes `Strict-Transport-Security: max-age=31536000; includeSubDomains`
   - response does not include `; preload` for this case.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_include_subdomains_invalid_env_falls_back_to_enabled_when_hsts_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and slight runtime-suite overhead.
- Improves confidence that malformed env values cannot weaken HSTS includeSubDomains posture.

## Next

1. Add HSTS preload invalid-env fallback coverage (`M38-S72`) to lock default preload-disabled semantics.
2. Continue security-header fallback hardening until all HSTS sub-control env toggles have explicit deterministic coverage.
