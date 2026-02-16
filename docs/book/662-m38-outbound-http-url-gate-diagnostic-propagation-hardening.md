# M38-S42 Outbound HTTP URL-Gate Diagnostic Propagation Hardening

## What it is

M38-S42 ensures HTTP net sinks preserve detailed URL-gate diagnostics instead of overwriting them with generic sink-level URL invalid errors.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`url.public(...)` and `url.internal(...)` now emit detailed IPv6 malformed-host diagnostics, but `httpClient.get*` wrappers still replaced those with generic `NET.URL_*_INVALID` responses. This slice keeps the most precise diagnostic visible.

## How it works

1. `sec4_rt_http_get` now only emits generic `NET.URL_PUBLIC_INVALID` when no detailed response is active.
2. `sec4_rt_http_get_internal` now only emits generic `NET.URL_INTERNAL_INVALID` when no detailed response is active.
3. Added runtime harness coverage that calls `httpClient.getInternal` with malformed IPv6 literals and asserts propagated URL-gate codes.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_url_gate_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_ipv6_literal_diagnostics_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Net sink wrappers now depend on active-response state to decide whether to emit fallback generic diagnostics.
- Error reporting is more precise and less surprising for operators.

## Next

1. Add request-parser split diagnostics for malformed IPv6-literal parse failures in `sec4_rt_http_get*` path.
2. Cover redirect/request parser malformed cases with dedicated `NET.REQUEST_*` assertions.
