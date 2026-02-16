# M38-S23 Outbound HTTP Redirect Configurable-Downgrade Policy Diagnostics

## What it is

M38-S23 extends redirect downgrade handling with an explicit configurable policy toggle while preserving secure default-deny behavior.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Some controlled environments may need temporary downgrade allowance for migration/debug, but secure-by-default runtime behavior must remain deterministic and deny downgrade unless explicitly enabled.

## How it works

1. Redirect resolver now accepts explicit policy input:
   - `allow_https_downgrade` boolean
2. Runtime follow-up reads policy env:
   - `SEC4_RT_NET_ALLOW_HTTPS_DOWNGRADE`
   - default: disabled (deny downgrade)
3. Behavior split:
   - deny mode: `https -> http` returns deterministic `NET.REDIRECT_DOWNGRADE_FORBIDDEN`
   - allow mode: resolver returns `OK` and normalized target URL

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_allows_https_to_http_downgrade_when_enabled_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime policy surface expands by one env toggle that must be controlled in production deployments.
- Secure default remains unchanged, but operators now have deterministic opt-in behavior for constrained migration scenarios.

## Next

1. Harden redirect policy env parsing bounds and deterministic failure diagnostics for invalid policy values.
2. Keep redirect hardening matrix tests green as policy surface grows.
