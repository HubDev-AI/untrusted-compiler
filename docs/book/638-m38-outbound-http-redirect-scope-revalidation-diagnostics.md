# M38-S18 Outbound HTTP Redirect Scope-Revalidation Diagnostics

## What it is

M38-S18 hardens redirect handling by splitting scope-revalidation failures into a dedicated deterministic runtime diagnostic.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Resolved redirect targets that violated net scope policy (public/internal boundaries) were previously surfaced through generic redirect-invalid behavior.

This slice makes scope-policy violations explicit and deterministic for incident/debug workflows.

## How it works

1. Redirect follow-up state now tracks scope-validation result separately from resolver status.
2. If redirect URL resolution succeeds but scope revalidation fails, runtime emits:
   - `NET.REDIRECT_SCOPE_INVALID`
3. Existing resolver-specific diagnostics remain unchanged:
   - `NET.REDIRECT_QUERY_INVALID`
   - `NET.REDIRECT_FRAGMENT_INVALID`
   - `NET.REDIRECT_TARGET_CHAR_INVALID`
   - `NET.REDIRECT_TARGET_INVALID`
   - `NET.REDIRECT_HOST_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scope_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Cross-scope redirects now fail with a stronger dedicated code rather than collapsing into generic redirect errors.
- This improves deterministic policy observability and makes production triage clearer.

## Next

1. Continue redirect hardening around revalidation-policy visibility and deterministic bypass diagnostics.
2. Keep redirect matrix tests green as new deterministic codes are introduced.
