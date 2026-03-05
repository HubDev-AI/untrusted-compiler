# M39: C Runtime String ABI Bridge for `log.*` Helpers

## Why

Core runtime log helpers still accepted raw `const char *` values while generated C code can pass tracked string handles in dynamic flows.

That mismatch risked runtime instability for log event names, keys, and text payloads in native execution paths.

## What Changed

1. Updated runtime ABI signatures to handle-based string inputs for:
   - `sec4_rt_log_event`
   - `sec4_rt_log_field` (key)
   - `sec4_rt_log_str`
   - `sec4_rt_log_redacted`
   - `sec4_rt_log_attr_redacted`
   - `sec4_rt_log_with_attr` (key)
   - `sec4_rt_log_with_http` (method/path)
2. Routed all string arguments in these helpers through `sec4_rt_resolve_tracked_or_literal_string(...)`.
3. Added deterministic defaults for missing/invalid resolved keys (`"field"`) and event names (`"event"`).
4. Updated C backend runtime signature assertions and hand-written runtime harness callsites to match the new handle-style ABI.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
2. `cargo test -p sec4 --test json_output c_bin_runtime_log_builders_emit_structured_json_when_clang_available`
3. `cargo test -p sec4 --test commands run_command_oneshot_supports_dynamic_err_internal_message`
