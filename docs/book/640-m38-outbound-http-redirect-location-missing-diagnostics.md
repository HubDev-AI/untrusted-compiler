# M38-S20 Outbound HTTP Redirect Location-Missing Diagnostics

## What it is

M38-S20 hardens redirect response handling by rejecting redirect status responses that omit the `Location` header with a dedicated deterministic runtime code.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The runtime already handled conflicting redirect location headers, but missing `Location` in redirect responses still flowed into generic redirect-invalid behavior.

This slice makes missing-location failures explicit and deterministic.

## How it works

1. In redirect follow-up flow, runtime now checks `redirect_location` before resolver execution.
2. If redirect status is present and `Location` is empty:
   - runtime emits `NET.REDIRECT_LOCATION_MISSING`
   - request fails without attempting redirect resolution.
3. Existing resolver/scope diagnostics remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_location_missing_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scope_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect handling is stricter for malformed upstream responses and now provides clearer diagnostics.
- This is intentional to keep outbound error classes deterministic and security-auditable.

## Next

1. Continue redirect hardening with deterministic scheme-downgrade diagnostics (`https -> http`).
2. Keep redirect matrix tests green while extending deterministic error coverage.
