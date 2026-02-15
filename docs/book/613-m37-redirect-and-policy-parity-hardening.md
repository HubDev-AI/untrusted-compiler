# M37-S1 Redirect And Policy Parity Hardening

## What it is

M37-S1 completes two implementation-first no-stub slices:

1. Policy persistence for active runtime keys:
   - `json.max_bytes`
   - `json.max_depth`
   - `net.public.max_redirects`
2. Runtime outbound redirect handling for `net.get`/`net.getInternal` with deterministic policy outcomes.

## Why it exists

No-stub alpha requires policy and runtime behavior to match on active execution paths.  
Before this slice, redirect behavior and policy-key persistence were incomplete for strict no-stub criteria.

## How it works

### Policy side

- The policy model now stores `json.max_bytes`, `json.max_depth`, and `net.public.max_redirects` as first-class persisted fields.
- Parsing and validation are deterministic and reject invalid values early.

### Runtime side

- Outbound HTTP runtime now handles redirect responses using runtime env controls:
  - `SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS` (default `false`)
  - `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS` (default `0`)
  - `SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS` (default `true`)
- Deterministic redirect failures are emitted for:
  - `NET.REDIRECT_FORBIDDEN`
  - `NET.REDIRECT_LIMIT`
  - `NET.REDIRECT_INVALID`
- Existing timeout/body-limit behavior remains intact.

## Validation

Policy:

- `cargo test -p sec4-core --test policy`

Runtime:

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_denied_by_default_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_limit_exceeded_when_clang_available`

## Trade-offs

- Redirect handling is deterministic and security-first, but still intentionally minimal for v0.1 runtime scope.
- TLS and deeper transport features remain bounded by current runtime backend capabilities.

## Next

1. M37-S2 runtime JSON semantic hardening.
2. M37-S3 middleware policy materialization hardening.
