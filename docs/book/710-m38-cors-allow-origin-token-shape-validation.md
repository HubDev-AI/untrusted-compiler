# M38-S87 CORS Allow-Origin Token Shape Validation

## What it is

M38-S87 hardens runtime CORS allow-origin policy validation so allowed-origin tokens must be syntactically valid origins (`http://...` / `https://...`) or wildcard (`*`).

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Earlier validation accepted generic header-safe text for allow-origin tokens. That allowed malformed values (for example missing scheme) to pass policy loading and later reach response behavior. Strict token-shape validation keeps runtime CORS policy deterministic and security-oriented.

## How it works

1. Added strict origin token validator:
   - allows wildcard (`*`),
   - requires `http`/`https` scheme for concrete origins,
   - validates authority host shape (including bracketed IPv6 literals),
   - validates optional numeric port range (`1..65535`),
   - rejects path/query/fragment in origin tokens.
2. Updated CORS allow-origin CSV validator to call the strict token validator per token.
3. Existing fallback behavior remains:
   - any invalid token in env policy makes the allow-origin policy fall back to wildcard (`*`).

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_allowed_origins_missing_scheme_token_falls_back_to_wildcard_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Validator logic is stricter and larger than generic header-value checks.
- Runtime policy now rejects previously tolerated malformed origin tokens; behavior is intentionally conservative and deterministic.

## Next

1. Continue middleware env-policy parity hardening with strict token validators where policy values represent structured network/security inputs.
2. Keep each hardening slice paired with focused runtime e2e regressions and roadmap/book traceability updates.
