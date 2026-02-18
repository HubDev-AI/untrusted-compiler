# M39: C Runtime String ABI Bridge for `auth.requireRole`, `sec.cspAdd`, and `sql.q`

## Why

Several core runtime helpers still accepted raw `const char *` parameters:

- `sql.q` query templates,
- `sec.cspAdd` directive/value strings,
- `auth.requireRole` required-role strings.

Generated/native flows can pass tracked string handles, so this left additional ABI mismatch points.

## What Changed

1. Updated runtime ABI signatures to handle-based string inputs for:
   - `sec4_rt_sql_q`
   - `sec4_rt_sec_csp_add`
   - `sec4_rt_auth_require_role`
2. Routed these helper string arguments through `sec4_rt_resolve_tracked_or_literal_string(...)`.
3. Added deterministic fallback for unresolved/empty `auth.requireRole` role values (`"role"`).
4. Updated C backend runtime signature assertions and affected runtime harness callsites in `json_output.rs`.
5. Updated auth-role runtime harness to source role dynamically from `sec4_rt_req_query("role")`, validating handle-based `auth.requireRole` path.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
2. `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_require_role_rejects_cookie_without_required_role_when_clang_available`
3. `cargo test -p sec4 --test json_output c_bin_http_runtime_err_with_helpers_enrich_error_response_when_clang_available`
4. `cargo test -p sec4 --test json_output c_bin_runtime_db_exec_query_one_roundtrip_returns_tracked_body_when_clang_available`
5. `cargo test -p sec4 --test commands run_command_oneshot_supports_dynamic_err_with_path`
