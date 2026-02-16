# M38-S43 Outbound HTTP Request-Parser IPv6 Diagnostics Hardening

## What it is

M38-S43 hardens outbound request-parser diagnostics so malformed bracketed IPv6 URLs emit deterministic split `NET.REQUEST_IPV6_*` codes across both direct request path and redirect-resolution path.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`sec4_rt_http_get*` already had request-parser split codes, but redirect resolution still collapsed malformed absolute IPv6 targets to generic redirect-invalid diagnostics. This slice keeps parser failure classes precise and stable.

## How it works

1. Extended redirect-resolve status model with parser-derived IPv6 classes:
   - `SEC4_RT_REDIRECT_RESOLVE_IPV6_BRACKET_MISSING`
   - `SEC4_RT_REDIRECT_RESOLVE_IPV6_EMPTY_LITERAL`
   - `SEC4_RT_REDIRECT_RESOLVE_IPV6_LITERAL_INVALID`
2. Updated `sec4_rt_resolve_redirect_url(...)` to forward parser status from `sec4_rt_parse_outbound_http_url(...)` instead of collapsing to generic invalid.
3. Updated outbound redirect handling in `sec4_rt_outbound_http_get_handle(...)` to emit:
   - `NET.REQUEST_IPV6_BRACKET_MISSING`
   - `NET.REQUEST_IPV6_EMPTY_LITERAL`
   - `NET.REQUEST_IPV6_LITERAL_INVALID`
4. Added harness coverage for both:
   - direct request parser path (`sec4_rt_outbound_http_get_handle(...)`)
   - redirect path through `sec4_rt_http_get_internal(...)`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect resolver/status plumbing is slightly larger due to explicit parser-to-resolver status mapping.
- Diagnostics become more granular, which improves debugging and policy triage consistency.

## Next

1. Add M38-S44 non-IPv6 request-parser fallback split diagnostics for cases still collapsing to `NET.URL_INVALID`.
2. Extend harness assertions to cover new fallback split codes end-to-end through `http_get*`.
