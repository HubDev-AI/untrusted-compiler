# M39: C Runtime String ABI Bridge for `err.with*` Helpers

## Why

After bridging top-level `err.*` helpers, `err.withPath`, `err.withDetail`, `err.withLimit`, and `err.withDependency` still accepted raw `const char *` string parameters.

That left dynamic tracked-string enrichment paths inconsistent and risky in native runtime execution.

## What Changed

1. Updated runtime ABI signatures to handle-based string inputs for:
   - `sec4_rt_err_with_path`
   - `sec4_rt_err_with_detail`
   - `sec4_rt_err_with_limit`
   - `sec4_rt_err_with_dependency`
2. Routed helper string arguments through `sec4_rt_resolve_tracked_or_literal_string(...)`.
3. Added deterministic fallback defaults when resolved strings are missing/empty:
   - detail key: `"detail"`
   - limit name: `"limit"`
   - dependency service: `"dependency"`
   - dependency operation: `"op"`
4. Updated C backend runtime signature assertions and runtime harness callsites in `json_output` to match the new handle-style ABI.
5. Added focused oneshot command coverage proving dynamic request-derived path values flow through `err.withPath(...)`.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
2. `cargo test -p sec4 --test json_output c_bin_http_runtime_err_with_helpers_enrich_error_response_when_clang_available`
3. `cargo test -p sec4 --test commands run_command_oneshot_supports_dynamic_err_with_path`
