# M38-S39 Outbound HTTP Client IPv6-Literal Parser Support

## What it is

M38-S39 adds bracketed IPv6-literal host support to the outbound HTTP URL parser used by runtime net client paths.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Internal allowlist checks gained IPv6 support in M38-S38, but outbound HTTP parser logic still handled only hostname/IPv4 forms. This slice removes that parser gap for client URL handling.

## How it works

1. Added bracketed-host parsing branch in `sec4_rt_parse_outbound_http_url`.
2. Preserved host normalization by storing the host value without brackets.
3. Kept deterministic invalid parsing for malformed bracketed hosts (missing closing bracket, empty literal).

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_url_parser_supports_ipv6_literals_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_roundtrip_succeeds_with_env_override_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Parser complexity increases with dual host-form handling.
- This slice validates parser behavior; full IPv6 transport roundtrip coverage is tracked as the next step.

## Next

1. Add real internal GET IPv6 loopback roundtrip coverage where environment supports `::1` binding.
2. Keep deterministic fallback/skip behavior when IPv6 loopback is unavailable.
