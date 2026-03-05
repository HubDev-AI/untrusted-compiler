# M39: C Runtime String ABI Bridge for `err.*` Helpers

## Why

Top-level `err.*` helpers in runtime C still used raw `const char *` parameters.

When dynamic tracked strings were passed from generated handlers (for example request-derived `err.internal(message)`), native runtime paths could terminate unexpectedly.

## What Changed

1. Updated runtime ABI signatures to handle-based string inputs for:
   - `sec4_rt_err_validation`
   - `sec4_rt_err_auth`
   - `sec4_rt_err_not_found`
   - `sec4_rt_err_conflict`
   - `sec4_rt_err_rate_limit`
   - `sec4_rt_err_internal`
2. Reused `sec4_rt_resolve_tracked_or_literal_string(...)` so error helpers support:
   - tracked dynamic values,
   - literal compatibility fallback where needed.
3. Updated C backend runtime signature assertions (`c_backend` test) for the new `int64_t` parameter forms.
4. Added focused command coverage proving dynamic `req.query(...)` values can flow into `err.internal(...)` in C oneshot runtime execution.
5. Updated affected runtime harness call sites in `json_output` tests to the explicit handle-style call shape where required.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
2. `cargo test -p sec4 --test commands run_command_oneshot_supports_dynamic_err_internal_message`
3. `cargo test -p sec4 --test json_output c_bin_http_runtime_err_internal_sets_error_response_when_clang_available`
4. `cargo test -p sec4 --test json_output c_bin_http_runtime_err_rate_limit_sets_limit_field_when_clang_available`
