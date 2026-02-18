# M39: LASM Auth Middleware Env-Off Parity

## Why

After adding LASM router middleware auth enforcement, there was still a policy/env precedence gap vs the C runtime:

- C runtime supports `SEC4_RT_AUTH_MODE=off`, which disables middleware auth checks.
- LASM runtime defaults were policy-only, so middleware auth remained enforced even when env mode was `off`.

## What Changed

1. Split LASM internal auth markers by source:
   - helper marker (`auth.require` / `auth.requireRole`)
   - middleware marker (`auth.withAuth`)
2. Added LASM effective auth-mode resolution with env override support:
   - `SEC4_RT_AUTH_MODE` now overrides policy mode when it contains a supported value.
3. Added LASM auth-cookie-name env override support:
   - `SEC4_RT_AUTH_COOKIE_NAME` now overrides policy cookie name when valid.
4. Middleware auth enforcement now bypasses when effective mode is `off`, while helper-based auth enforcement remains active.
5. Internal auth middleware markers are stripped before response emission.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_auth_middleware_when_auth_mode_env_off`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_require_role_with_ctx_current`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware`
