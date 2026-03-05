# M38-S80 Auth Cookie-Name Invalid Env Fallback

## What it is

M38-S80 hardens runtime cookie-auth config handling so invalid `SEC4_RT_AUTH_COOKIE_NAME` values fall back to the default session cookie name.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Cookie-auth relies on exact cookie-name matching. If env configuration injects an invalid cookie name, runtime can reject valid cookie-auth requests unexpectedly. This slice makes cookie-name resolution deterministic and safe under malformed env input.

## How it works

1. Updated runtime auth cookie-name resolution:
   - read `SEC4_RT_AUTH_COOKIE_NAME`
   - accept only valid token syntax
   - fallback to `"session"` when invalid/empty.
2. Added clang-gated e2e coverage:
   - `c_bin_http_runtime_auth_cookie_name_invalid_env_falls_back_to_session_cookie`
   - sets:
     - `SEC4_RT_AUTH_MODE=cookie`
     - `SEC4_RT_AUTH_COOKIE_NAME=session;invalid`
   - sends `Cookie: session=session123`
   - asserts deterministic `HTTP/1.1 200 OK`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_cookie_name_invalid_env_falls_back_to_session_cookie`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_session_cookie_when_cookie_auth_mode_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_require_role_rejects_cookie_without_required_role_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one env-validation check and one focused e2e runtime test.
- Slightly increases test runtime but eliminates a real misconfiguration failure mode for cookie-auth flows.

## Next

1. Continue auth/CSRF env fallback hardening for remaining fields so malformed env config cannot silently degrade runtime security behavior.
2. Keep pairing runtime changes with e2e assertions and roadmap/book traceability updates.
