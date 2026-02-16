# M38-S36 Outbound HTTP Public-URL Policy-List Env-Validation Diagnostics

## What it is

M38-S36 adds strict runtime validation for public-url policy list env keys (`allowed_schemes`, `allowed_domains`, `blocked_domains`, `allowed_ports`) with deterministic field-specific diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, malformed list tokens in public-url env policy keys could be silently ignored. That made runtime behavior less predictable and harder to debug in operator environments.

## How it works

1. Added strict validators for public-url policy lists:
   - scheme list (`http|https` only)
   - domain lists (host-token validation)
   - port list (numeric `1..65535`)
2. Added deterministic diagnostics with `policyKey` details for each invalid list key:
   - `NET.URL_PUBLIC_POLICY_ALLOWED_SCHEMES_INVALID`
   - `NET.URL_PUBLIC_POLICY_ALLOWED_DOMAINS_INVALID`
   - `NET.URL_PUBLIC_POLICY_BLOCKED_DOMAINS_INVALID`
   - `NET.URL_PUBLIC_POLICY_ALLOWED_PORTS_INVALID`
3. Kept valid list allow/deny behavior unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_enforces_allowed_ports_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- URL policy path now performs additional validation branches.
- The stricter parser fails misconfigured env values early, which can surface pre-existing config issues immediately.

## Next

1. Apply the same strict malformed-list diagnostics pattern to internal-url allowlist env keys.
2. Keep policy error envelope shape and `policyKey` details stable across public/internal URL policy paths.
