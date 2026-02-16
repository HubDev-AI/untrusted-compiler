# M38-S86 CORS Wildcard Credentials Runtime Guard

## What it is

M38-S86 adds a runtime CORS safety guard: when effective allow-origin is wildcard (`*`), runtime suppresses `Access-Control-Allow-Credentials: true` even if credentials are enabled via env policy.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Wildcard origin plus credentials is an unsafe CORS combination. Static policy/audit can flag it, but runtime env policy should also enforce safe behavior so misconfiguration cannot emit insecure response headers.

## How it works

1. Updated success/preflight CORS header assembly to resolve wildcard allow-origin before emitting headers.
2. Added wildcard token detection for wildcard appearing anywhere in CSV allow-origin policy values.
3. Added credentials emission guard:
   - emit `Access-Control-Allow-Credentials: true` only when effective allow-origin is non-wildcard.
4. Preserved non-wildcard behavior:
   - credentials header still appears when allow-origin is concrete and credentials env policy is enabled.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_wildcard_with_allow_credentials_env_suppresses_credentials_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_non_wildcard_with_allow_credentials_env_emits_credentials_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime now has one additional CORS guard branch and wildcard-token detection step.
- Behavior is intentionally strict: env values asking for wildcard+credentials are clamped to safe header output.

## Next

1. Continue closing remaining env-policy runtime parity gaps with deterministic fallback semantics.
2. Keep policy hardening changes paired with concrete runtime e2e tests for both success and reject/error paths.
