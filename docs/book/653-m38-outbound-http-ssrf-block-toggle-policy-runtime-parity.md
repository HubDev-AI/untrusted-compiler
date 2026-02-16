# M38-S33 Outbound HTTP SSRF Block-Toggle Policy/Runtime Parity

## What it is

M38-S33 implements policy-to-runtime parity for SSRF block toggles so `sec4.policy` controls how `url.public(...)` treats private ranges, loopback, link-local, and metadata IPs.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`net.ssrf` already exposed these keys in policy syntax, but only `resolve_dns`/`revalidate_redirects` were persisted and bridged. That created policy/runtime drift for core SSRF controls.

## How it works

1. Extended `NetSsrfPolicyConfig` with persisted booleans:
   - `block_private_ranges`
   - `block_loopback`
   - `block_link_local`
   - `block_metadata_ips`
   Defaults remain secure (`true`).
2. Updated policy parsing to store those values when present.
3. `sec4 run` now exports the corresponding runtime env keys:
   - `SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES`
   - `SEC4_RT_NET_SSRF_BLOCK_LOOPBACK`
   - `SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL`
   - `SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS`
4. Runtime public-URL validation now applies these toggles to:
   - direct IPv4 host classification,
   - IPv6/v4-mapped DNS-resolved classification,
   - metadata address handling (`169.254.169.254`) independently from generic link-local handling.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available`
- `cargo test -p sec4 --test commands run_command_oneshot_allows_public_loopback_when_ssrf_block_toggles_are_disabled_by_policy`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- More runtime policy branches increase code complexity in URL classification logic.
- Keeping secure defaults while adding disable-toggles supports local/dev workflows, but broad disablement can weaken SSRF posture if misconfigured.

## Next

1. Add strict env-value validation diagnostics for these new SSRF block-toggle keys.
2. Emit deterministic policy-invalid error envelopes for malformed toggle values, matching the redirect-policy diagnostics style.
