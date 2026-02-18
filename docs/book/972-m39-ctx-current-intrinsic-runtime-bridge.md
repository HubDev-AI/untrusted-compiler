# M39: `ctx.current` Intrinsic Runtime Bridge

## Why

`auth.require(...)` and `auth.requireRole(...)` require a `Ctx` value, but route handlers did not have a first-class intrinsic to fetch the active request context.

That gap forced awkward fixtures and prevented clean auth-helper usage from normal handler bodies.

## What Changed

1. Added a new intrinsic:
   - `ctx.current()` (alias: `ctx_current()`)
   - return type: `Ctx`
   - effect: `net`
2. Added strict call-shape enforcement:
   - `ctx.current` expects zero arguments.
3. Added browser-profile fence for `ctx.current`:
   - deterministic `E2002` policy diagnostic in `profile = "browser"`.
4. Added C backend lowering bridge:
   - intrinsic placeholder -> `sec4_rt_ctx_current(...)`.
5. Added runtime ABI + implementation:
   - `runtime/c/sec4_runtime.h`: `int64_t sec4_rt_ctx_current(void);`
   - `runtime/c/sec4_runtime.c`: returns request-scoped deterministic context handle when request exists; falls back to base context handle otherwise.
6. Updated auth intrinsic coverage fixtures to use `ctx.current()` in generated/compiled paths.
7. Added real `sec4 run --oneshot` integration coverage proving handler-level `ctx.current()` + `auth.requireRole(...)` succeeds under valid authorization input.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_rewrites_auth_requirement_intrinsics_to_runtime_symbols`
2. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
3. `cargo test -p sec4 --test json_output build_emit_c_bin_handles_auth_requirement_intrinsics_when_clang_available`
4. `cargo test -p sec4 --test commands run_command_oneshot_supports_ctx_current_with_auth_require_role`
