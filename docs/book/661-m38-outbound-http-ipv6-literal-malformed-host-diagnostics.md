# M38-S41 Outbound HTTP IPv6-Literal Malformed-Host Diagnostics

## What it is

M38-S41 adds deterministic malformed IPv6-literal diagnostics for both `url.public(...)` and `url.internal(...)` gate paths.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After IPv6-literal parser/transport support, malformed bracketed host inputs still collapsed into generic gate failures. This slice makes those invalid classes explicit and actionable.

## How it works

1. Added a runtime bracketed-IPv6 host classifier used by URL gates.
2. Public URL gate now emits:
   - `NET.URL_PUBLIC_IPV6_BRACKET_MISSING`
   - `NET.URL_PUBLIC_IPV6_EMPTY_LITERAL`
   - `NET.URL_PUBLIC_IPV6_LITERAL_INVALID`
3. Internal URL gate now emits:
   - `NET.URL_INTERNAL_IPV6_BRACKET_MISSING`
   - `NET.URL_INTERNAL_IPV6_EMPTY_LITERAL`
   - `NET.URL_INTERNAL_IPV6_LITERAL_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_ipv6_literal_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- URL gate path contains additional classification logic.
- Diagnostic surface grows, but operators now get precise invalid-input reasons for IPv6-literal host errors.

## Next

1. Split malformed IPv6-literal request diagnostics in outbound GET parser path (`sec4_rt_http_get*`) from generic request-invalid errors.
2. Keep valid IPv6 transport behavior unchanged while hardening malformed-request diagnostics.
