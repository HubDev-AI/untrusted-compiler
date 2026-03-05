# M38-S78 Auth Mode Invalid Env Fallback

## What it is

M38-S78 hardens runtime auth-mode env handling so unsupported `SEC4_RT_AUTH_MODE` values deterministically fall back to `token`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, unsupported auth-mode env values could leave runtime in an undefined auth-mode path, which rejected valid Authorization-header requests. We need deterministic fallback behavior so malformed env values cannot break auth mode semantics.

## How it works

1. Added runtime auth-mode normalization helpers:
   - supported values: `off`, `token`, `cookie`, `mixed`
   - unsupported/empty values clamp to `token`.
2. Applied normalization in:
   - auth policy loading (`sec4_rt_load_auth_policy_from_env`)
   - effective auth-mode resolution (`sec4_rt_effective_auth_mode`)
3. Added clang-gated e2e test:
   - `c_bin_http_runtime_auth_mode_invalid_env_falls_back_to_token_mode`
   - sets `SEC4_RT_AUTH_MODE=MAYBE`
   - sends a valid `Authorization` header
   - asserts deterministic `HTTP/1.1 200 OK`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_mode_invalid_env_falls_back_to_token_mode`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_auth_header_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_session_cookie_when_cookie_auth_mode_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one small runtime normalization path and one focused e2e test.
- Keeps env-policy behavior deterministic and avoids silent auth-mode misconfiguration regressions.

## Next

1. Continue middleware env-hardening on remaining auth/CSRF fields (`SEC4_RT_CSRF_*`, auth cookie-related toggles) with deterministic fallback contracts.
2. Keep runtime hardening slices paired with explicit e2e tests and roadmap/book updates.
