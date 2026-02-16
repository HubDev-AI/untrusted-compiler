# M38-S82 CSRF Name Env Policy Materialization

## What it is

M38-S82 adds real runtime materialization for CSRF header/cookie names and deterministic fallback behavior for malformed env inputs.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, CSRF enforcement and token-issue response headers used hardcoded names (`X-CSRF-Token`, `csrf`). Policy/env-driven name customization was not honored, and malformed inputs had no explicit clamp path. This blocked valid customization and weakened deterministic config behavior.

## How it works

1. Extended CSRF policy/router runtime state with:
   - `csrf_header_name`
   - `csrf_cookie_name`
2. Added env materialization for:
   - `SEC4_RT_CSRF_HEADER_NAME`
   - `SEC4_RT_CSRF_COOKIE_NAME`
3. Added deterministic validation/fallback:
   - invalid names clamp to defaults:
     - header: `X-CSRF-Token`
     - cookie: `csrf`
4. Updated runtime enforcement + issue-token paths:
   - CSRF request checks use configured/fallback names.
   - `csrf.issueToken` emits matching configured/fallback names in response headers.
5. Added clang-gated e2e coverage for:
   - valid custom names success path
   - invalid-name fallback to defaults.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_custom_csrf_names_from_env`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_csrf_names_invalid_env_fall_back_to_defaults`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled`
- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_csrf_issue_token_intrinsic_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds small state/validation complexity in runtime CSRF policy handling.
- Increases e2e test count, but closes a real policy-materialization gap and keeps behavior deterministic.

## Next

1. Continue auth/CSRF/security middleware env materialization hardening for remaining policy fields where runtime behavior is still partially fixed/defaulted.
2. Keep pairing each runtime change with e2e assertions and roadmap/book updates.
