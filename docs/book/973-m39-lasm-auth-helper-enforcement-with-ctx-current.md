# M39: LASM Auth-Helper Enforcement with `ctx.current()`

## Why

LASM run-mode response planning already extracted route/response behavior, but handler auth helper calls (`auth.require`, `auth.requireRole`) were not enforced at request time.

That created backend drift: security-relevant helper calls in handlers could be ignored in `sec4 run --backend lasm`.

## What Changed

1. Extended LASM route plan extraction to collect auth requirements from handler call graphs:
   - `auth.require(...)` marks route as auth-required.
   - `auth.requireRole(...)` marks route as auth-required with a required role.
2. Added request-time LASM auth enforcement in dynamic response materialization:
   - unauthorized -> deterministic `401` (`AUTH.UNAUTHORIZED`)
   - missing required role -> deterministic `403` (`AUTH.FORBIDDEN`)
3. Added dynamic required-role materialization support using existing request placeholder pipeline (for example role derived from `req.query(...)`).
4. Added LASM auth policy defaults to runtime header defaults (`auth_mode`, `auth_cookie_name`) so enforcement follows active policy mode semantics (`token` / `cookie` / `mixed` / `off` fallback behavior).
5. Preserved external response contracts by using internal auth markers that are removed before response emission.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_require_role_with_ctx_current`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_serves_request_and_exits`
