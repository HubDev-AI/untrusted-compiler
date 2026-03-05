# M39: LASM CSRF Middleware Env-Off Parity

## Why

LASM csrf middleware enforcement was policy-only, while the C runtime also allows env-driven csrf overrides (`SEC4_RT_CSRF_*`).

That mismatch meant LASM behavior could diverge from C runtime when operators configured runtime csrf settings through environment variables.

## What Changed

1. Added LASM env-bool parsing for deterministic runtime overrides.
2. Added LASM effective csrf-enabled resolution with env precedence:
   - `SEC4_RT_CSRF_ENABLED`
   - `SEC4_RT_CSRF_MODE` (`off` forces disabled)
3. Added LASM effective csrf header/cookie name resolution with env overrides:
   - `SEC4_RT_CSRF_COOKIE_NAME`
   - `SEC4_RT_CSRF_HEADER_NAME`
4. Added LASM effective protected-method resolution with env override:
   - `SEC4_RT_CSRF_PROTECTED_METHODS`
5. CSRF middleware request-time enforcement now uses these effective values.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_csrf_middleware_when_csrf_mode_env_off`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_auth_middleware_when_auth_mode_env_off`
