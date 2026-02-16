# M38-S21 Outbound HTTP Redirect Downgrade Diagnostics

## What it is

M38-S21 hardens redirect policy by forbidding `https -> http` redirect downgrades with deterministic resolver and runtime diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Security posture for outbound redirects should prevent transport downgrade from encrypted to plaintext targets when following redirect chains.

This slice introduces deterministic downgrade rejection so this class is explicit and policy-auditable.

## How it works

1. Redirect resolver tracks current URL scheme.
2. If a resolved redirect target downgrades from `https` to `http`, resolver returns:
   - `SEC4_RT_REDIRECT_RESOLVE_DOWNGRADE_INVALID`
3. Redirect follow-up maps this status to deterministic runtime code:
   - `NET.REDIRECT_DOWNGRADE_FORBIDDEN`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_location_missing_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Downgrade redirects are now explicitly rejected even when other redirect settings are permissive.
- This is stricter by design and aligns with secure-by-default outbound policy.

## Next

1. Add openssl-gated end-to-end runtime-path coverage for downgrade diagnostics.
2. Keep redirect/parser matrix tests green while broadening deterministic policy diagnostics.
