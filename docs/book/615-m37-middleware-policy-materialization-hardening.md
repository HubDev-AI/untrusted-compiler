# M37-S3 Middleware Policy Materialization Hardening

## What it is

M37-S3 turns middleware `fromPolicy()` usage into real runtime behavior for CORS, security headers, CSRF, and auth-mode routing context:

1. `sec4 run` now exports middleware-relevant policy keys into runtime env.
2. Runtime `with*` middleware hooks consume policy handles and apply materialized values.
3. Response emission uses policy-driven dynamic headers instead of fixed constants.

## Why it exists

Before this slice, middleware policy hooks were only partially connected:

- runtime had policy-handle stubs for middleware configuration,
- but `sec4 run` was not exporting middleware policy values,
- and header behavior stayed mostly static.

No-stub alpha needs policy-to-runtime parity so `sec4.policy` controls live serving behavior deterministically.

## How it works

### Runtime middleware state materialization

`runtime/c/sec4_runtime.c` now carries policy-materialized router state for:

- CORS:
  - `cors_allow_origin`
  - `cors_allow_credentials`
  - `cors_require_vary_origin`
  - `cors_allow_methods`
  - `cors_allow_headers`
  - `cors_max_age_seconds`
- Security headers:
  - `security_x_content_type_options`
  - `security_x_frame_options`
  - `security_referrer_policy`
- CSRF:
  - `csrf_mode`
  - `csrf_protected_methods`
- Auth:
  - `auth_mode`

`cors.fromPolicy()`, `sec.defaultHeaders()`, `csrf.fromPolicy()`, and `auth.fromPolicy()` now load env-backed policy snapshots and return dedicated policy handles consumed by `with*` middleware functions.

### Dynamic header and CSRF method behavior

- CORS normal/preflight header blocks are now built from router policy state.
- `Vary: Origin` is emitted when required and origin is non-wildcard.
- Security headers are emitted from router policy state and can be fully disabled by policy.
- CSRF protected-method checks now honor policy-provided method lists instead of hardcoded defaults only.

### CLI policy bridge in `sec4 run`

`compiler/sec4-cli/src/main.rs` now exports middleware policy env keys before launching c-bin runtime:

- `SEC4_RT_CORS_*`
- `SEC4_RT_SECURITY_HEADERS_*`
- `SEC4_RT_CSRF_*`
- `SEC4_RT_AUTH_MODE`

This slice also bridges existing runtime controls from policy for parity:

- `SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS`
- `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS`
- `SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS`
- `SEC4_RT_JSON_MAX_BYTES`
- `SEC4_RT_JSON_MAX_DEPTH`

## Validation

- `cargo test -p sec4 --test commands`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_custom_header_and_cookie_when_set`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

New run-command integration coverage in `commands.rs`:

- `run_command_oneshot_applies_cors_from_policy`
- `run_command_oneshot_disables_security_headers_from_policy`
- `run_command_oneshot_disables_csrf_from_policy`

## Trade-offs

- CORS method/header lists are still defaulted at runtime in this slice because policy model coverage for those fields is not yet expanded.
- Auth-mode materialization is now wired, but deeper auth principal/session semantics remain future runtime work.

## Next

1. M37-S4 structured runtime log emission path.
2. Final no-stub alpha verification pass (full alpha smoke + targeted runtime harness set).
3. Alpha publish checklist delta update from verification evidence.
