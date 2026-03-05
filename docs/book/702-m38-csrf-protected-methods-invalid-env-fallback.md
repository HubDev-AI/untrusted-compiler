# M38-S79 CSRF Protected Methods Invalid Env Fallback

## What it is

M38-S79 hardens runtime CSRF policy loading so invalid `SEC4_RT_CSRF_PROTECTED_METHODS` values fall back to the default unsafe-method set (`POST,PUT,PATCH,DELETE`).

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CSRF protection is method-gated. If the env-provided protected-method list is malformed, runtime must not accidentally disable unsafe-method coverage. This slice enforces deterministic fallback so invalid policy tokens cannot weaken CSRF posture.

## How it works

1. Added runtime helper to validate whether a configured protected-method list contains at least one recognized unsafe verb.
2. During CSRF policy loading:
   - read `SEC4_RT_CSRF_PROTECTED_METHODS`
   - if the list has no valid unsafe verbs, clamp to `POST,PUT,PATCH,DELETE`.
3. Added clang-gated e2e coverage:
   - `c_bin_http_runtime_csrf_protected_methods_invalid_env_falls_back_to_default_set`
   - sets `SEC4_RT_CSRF_PROTECTED_METHODS=MAYBE`
   - sends `POST /users` without CSRF tokens
   - asserts deterministic `403` + CSRF error envelope.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_csrf_protected_methods_invalid_env_falls_back_to_default_set`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one small normalization helper and one focused e2e test.
- Slightly increases runtime-suite duration, but closes a real policy-misconfiguration hardening gap.

## Next

1. Continue auth/CSRF env-hardening for remaining mode/toggle fields with deterministic fallback contracts.
2. Keep each hardening slice paired with e2e verification plus roadmap/book traceability updates.
