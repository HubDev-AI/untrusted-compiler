# M38-S35 Outbound HTTP SSRF Resolve-DNS Env-Validation Diagnostics

## What it is

M38-S35 adds strict runtime validation for `SEC4_RT_NET_SSRF_RESOLVE_DNS` in the `url.public(...)` policy path, with deterministic policy diagnostics on malformed values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

SSRF block toggles were hardened in M38-S34, but `resolve_dns` still accepted malformed values via fallback behavior. This slice closes that gap and keeps SSRF policy env controls consistent.

## How it works

1. Replaced fallback parse path for `SEC4_RT_NET_SSRF_RESOLVE_DNS` with strict boolean parsing.
2. On invalid values, runtime now emits:
   - `NET.SSRF_POLICY_RESOLVE_DNS_INVALID`
   - with `details:[{"key":"policyKey","value":"SEC4_RT_NET_SSRF_RESOLVE_DNS"}]`
3. Existing resolve-dns enabled/disabled behavior remains unchanged for valid values.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_resolve_dns_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_dns_resolution_toggle_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Strict parsing is less forgiving for malformed environments, but this is intentional fail-closed behavior.
- Error surface grows by one code, improving operational debugging precision.

## Next

1. Apply strict malformed-token diagnostics to public-url policy list env keys (`ALLOWED_SCHEMES`, `ALLOWED_DOMAINS`, `BLOCKED_DOMAINS`, `ALLOWED_PORTS`).
2. Keep policy-detail envelope shape consistent across all runtime net policy validation paths.
