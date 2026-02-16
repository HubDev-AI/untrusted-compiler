# M38-S29 Outbound HTTP Redirect Policy Structured-Details Diagnostics

## What it is

M38-S29 hardens redirect policy diagnostics by adding deterministic structured `details` fields that identify the invalid policy key in runtime error envelopes.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Field-specific error codes help, but machine-friendly debugging and alert routing also need deterministic structured context. This slice adds stable `details` for redirect policy invalid paths.

## How it works

1. Added runtime helper to emit standard error envelope with one structured detail entry.
2. Redirect policy invalid paths now emit:
   - field-specific code (existing from S27)
   - `details:[{"key":"policyKey","value":"<ENV_NAME>"}]`
3. Extended harness assertions to verify both `code` and structured `details` content.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allow_downgrade_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Error envelope payloads are slightly larger due to structured detail metadata.
- The additional bytes are acceptable for improved deterministic observability.

## Next

1. Stabilize redirect diagnostics execution flow with deterministic test orchestration and no compile-lock contention patterns.
2. Keep redirect hardening matrix green while preparing next alpha-no-stub verification pass.
