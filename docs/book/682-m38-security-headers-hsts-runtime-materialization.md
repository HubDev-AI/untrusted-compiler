# M38-S61 Security-Headers HSTS Runtime Materialization

## What it is

M38-S61 implements real HSTS header materialization in runtime security headers.

Runtime now carries HSTS fields in security-header policy/router state and emits `Strict-Transport-Security` deterministically when HSTS is enabled.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Security headers runtime already emitted `nosniff`, `x-frame-options`, `referrer-policy`, and (after M38-S60) CSP. HSTS was still missing from runtime materialization.

This slice closes that gap with policy-driven HSTS rendering.

## How it works

1. Extended runtime security-header policy/router state with HSTS fields:
   - `hsts_enabled`
   - `hsts_max_age_seconds`
   - `hsts_include_subdomains`
   - `hsts_preload`
2. Added env-policy loading for HSTS controls:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD`
3. Added deterministic HSTS header rendering in `sec4_rt_security_headers_block`:
   - `Strict-Transport-Security: max-age=<n>[; includeSubDomains][; preload]`
4. Added e2e runtime coverage:
   - `c_bin_http_runtime_applies_security_headers_hsts_when_enabled`

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds more policy fields and header serialization logic in runtime middleware path.
- Keeps HSTS behavior deterministic and env-driven without introducing runtime HTTP/TLS-mode auto-detection complexity.

## Next

1. Add suffixed-token fallback coverage for `SEC4_RT_ALLOW_INTERNAL_NET` (`M38-S62`) to continue strict token-boundary hardening.
2. Add a targeted invalid HSTS max-age env fallback test (non-integer/negative values) for policy parse resilience.
