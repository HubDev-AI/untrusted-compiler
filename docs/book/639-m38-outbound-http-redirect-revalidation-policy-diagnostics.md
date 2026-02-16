# M38-S19 Outbound HTTP Redirect Revalidation-Policy Diagnostics

## What it is

M38-S19 hardens redirect policy behavior by making cross-scope redirect handling deterministic even when redirect scope revalidation is explicitly disabled.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The runtime already had a revalidation toggle for redirect scope checks, but cross-scope outcomes under disabled revalidation were not explicitly surfaced as a dedicated deterministic policy diagnostic.

This slice keeps behavior fail-closed and auditable for cross-scope redirects.

## How it works

1. Redirect follow-up scope is now evaluated deterministically before follow-up request.
2. If resolved redirect is cross-scope and revalidation toggle is disabled:
   - runtime emits `NET.REDIRECT_SCOPE_REVALIDATION_DISABLED`
   - request fails without following the redirect.
3. Existing scope diagnostics remain:
   - `NET.REDIRECT_SCOPE_INVALID` for normal revalidation-enabled scope failures.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scope_revalidation_disabled_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scope_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now blocks an additional risky redirect scenario under disabled revalidation and emits a dedicated policy-facing diagnostic.
- This slightly tightens behavior but preserves deterministic security posture and easier policy triage.

## Next

1. Continue redirect hardening with deterministic downgrade diagnostics for `https -> http` redirect chains.
2. Keep redirect matrix tests green as policy diagnostics are expanded.
