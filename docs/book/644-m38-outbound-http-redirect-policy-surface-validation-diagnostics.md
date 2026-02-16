# M38-S24 Outbound HTTP Redirect Policy-Surface Validation Diagnostics

## What it is

M38-S24 hardens redirect policy surface by validating redirect-related env policy values strictly and emitting deterministic diagnostics when policy values are malformed.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect policy env parsing previously used fallback behavior for malformed values, which could silently mask policy misconfiguration.

This slice ensures malformed redirect policy values fail closed with explicit diagnostics.

## How it works

1. Added strict redirect-policy env parsers:
   - strict bool parser for policy flags
   - bounded non-negative integer parser for redirect hop settings
2. Redirect flow now validates:
   - `SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS`
   - `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS`
   - `SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS`
   - `SEC4_RT_NET_ALLOW_HTTPS_DOWNGRADE`
3. Invalid policy values emit deterministic runtime code:
   - `NET.REDIRECT_POLICY_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_allows_https_to_http_downgrade_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect runtime now fails fast on malformed policy values instead of defaulting silently.
- This increases strictness but improves deterministic security posture and operator visibility.

## Next

1. Harden hop-budget accounting and policy cap diagnostics for redirect chains.
2. Keep redirect hardening matrix tests green as policy and diagnostics tighten further.
