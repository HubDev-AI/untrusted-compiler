# M38-S34 Outbound HTTP SSRF Block-Toggle Env-Validation Diagnostics

## What it is

M38-S34 hardens runtime SSRF block-toggle handling by adding strict env-value validation and field-specific deterministic diagnostics for malformed toggle values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After SSRF block-toggle policy/runtime parity (M38-S33), malformed env values could still silently fall back to defaults. This slice makes invalid runtime policy surface explicit and auditable.

## How it works

1. `url.public(...)` runtime validation now parses these env keys with strict boolean parsing:
   - `SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES`
   - `SEC4_RT_NET_SSRF_BLOCK_LOOPBACK`
   - `SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL`
   - `SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS`
2. Invalid values now emit dedicated deterministic diagnostics with structured detail:
   - `NET.SSRF_POLICY_BLOCK_PRIVATE_RANGES_INVALID`
   - `NET.SSRF_POLICY_BLOCK_LOOPBACK_INVALID`
   - `NET.SSRF_POLICY_BLOCK_LINK_LOCAL_INVALID`
   - `NET.SSRF_POLICY_BLOCK_METADATA_IPS_INVALID`
   - `details:[{"key":"policyKey","value":"<ENV_KEY>"}]`
3. `sec4_rt_url_public(...)` now preserves active detailed runtime policy errors instead of overwriting them with generic `NET.URL_PUBLIC_INVALID`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_dns_resolution_toggle_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime policy parsing path is more explicit and larger.
- Strict parsing can surface misconfigured environments earlier, which is safer but may require operators to fix previously ignored values.

## Next

1. Apply the same strict env-validation diagnostic pattern to `SEC4_RT_NET_SSRF_RESOLVE_DNS`.
2. Keep error-envelope shape and policy-key details consistent across all SSRF env controls.
