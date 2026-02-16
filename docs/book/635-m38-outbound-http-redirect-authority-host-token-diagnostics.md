# M38-S15 Outbound HTTP Redirect-Authority Host-Token Diagnostics

## What it is

M38-S15 hardens redirect authority handling by validating host tokens and canonicalizing resolved redirect authorities.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect resolution previously accepted broader authority host payload and lacked a dedicated diagnostic for invalid host tokens.

This slice makes authority behavior deterministic and explicit.

## How it works

1. Redirect resolver host-token validation:
   - enforces strict host label characters and label boundaries,
   - rejects invalid authority hosts with deterministic `NET.REDIRECT_HOST_INVALID`.
2. Redirect authority normalization:
   - lowercases resolved host values,
   - elides default ports in canonical rebuilt redirect URLs.
3. Existing redirect target normalization and scope revalidation stay unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_absolute_redirect_upper_host_succeeds_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_host_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_relative_redirect_is_normalized_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects more malformed redirect authorities that previously could collapse into generic redirect-invalid behavior.
- This is intentional for deterministic diagnostics and safer outbound redirect handling.

## Next

1. Continue outbound parser hardening around redirect query/fragment target diagnostics.
2. Keep focused redirect and parser tests green per slice.
