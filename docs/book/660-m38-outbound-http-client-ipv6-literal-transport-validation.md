# M38-S40 Outbound HTTP Client IPv6-Literal Transport Validation

## What it is

M38-S40 adds real runtime integration coverage for internal GET requests against IPv6 loopback literal URLs (`http://[::1]:port/...`).

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S39 added parser-level IPv6-literal support. This slice verifies the transport path (`sec4_rt_http_get_internal`) can execute a real IPv6 loopback roundtrip where the environment supports it.

## How it works

1. Added clang-gated runtime integration test:
   - `c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
2. Test binds a one-shot IPv6 loopback server on `[::1]:0`, calls `sec4_rt_http_get_internal(...)` with bracketed URL, and asserts success.
3. If IPv6 loopback bind is unavailable, the test deterministically skips with an explicit message.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_url_parser_supports_ipv6_literals_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime behavior is environment-dependent for IPv6 loopback availability.
- Skip behavior is explicit to keep CI deterministic while still validating IPv6 transport where supported.

## Next

1. Add deterministic malformed IPv6-literal diagnostics in URL gate paths.
2. Keep valid IPv6 parser/transport paths green while expanding invalid-class coverage.
