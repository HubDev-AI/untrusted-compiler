# M38-S37 Outbound HTTP Internal-URL Allowlist Env-Validation Diagnostics

## What it is

M38-S37 adds strict runtime validation for internal-url allowlist env keys (`allowed_domains`, `allowed_cidrs`) and emits deterministic field-specific diagnostics for malformed values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Public-url policy env validation was already hardened (M38-S36), but internal allowlist env parsing still tolerated malformed values. This slice closes that consistency gap.

## How it works

1. Added strict validators for internal allowlist env lists:
   - domain list validation
   - IPv4 CIDR list validation
2. Added deterministic diagnostics with `policyKey` details:
   - `NET.URL_INTERNAL_POLICY_ALLOWED_DOMAINS_INVALID`
   - `NET.URL_INTERNAL_POLICY_ALLOWED_CIDRS_INVALID`
3. Updated `url.internal(...)` to preserve active detailed policy diagnostics instead of replacing them with generic `NET.URL_INTERNAL_INVALID`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Additional validation branches increase runtime policy-check complexity.
- Fail-closed behavior may surface configuration problems immediately where fallback behavior previously masked them.

## Next

1. Extend internal allowlist CIDR support to IPv6 CIDR tokens.
2. Keep malformed-CIDR diagnostic behavior deterministic and aligned with policy-key detail envelopes.
