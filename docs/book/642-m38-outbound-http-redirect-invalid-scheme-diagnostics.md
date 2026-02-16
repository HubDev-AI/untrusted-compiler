# M38-S22 Outbound HTTP Redirect Invalid-Scheme Diagnostics

## What it is

M38-S22 hardens redirect resolution by introducing deterministic diagnostics for absolute redirect targets that use unsupported schemes.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Absolute redirect targets with non-HTTP schemes (for example `ftp://...`) should not be treated as relative targets or generic invalid payload. They require a dedicated deterministic error class for policy/debug clarity.

## How it works

1. Redirect resolver now detects absolute `://` scheme targets before relative-path merge logic.
2. If scheme is not `http` or `https`, resolver returns:
   - `SEC4_RT_REDIRECT_RESOLVE_SCHEME_INVALID`
3. Redirect follow-up maps this resolver status to deterministic runtime code:
   - `NET.REDIRECT_SCHEME_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_location_missing_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime classification becomes stricter and more explicit for absolute redirect targets.
- This is intentional to preserve deterministic security diagnostics and reduce ambiguity in redirect incident triage.

## Next

1. Add configurable redirect downgrade policy toggles with deterministic diagnostics for stricter deployment modes.
2. Keep redirect hardening matrix tests green as policy controls expand.
