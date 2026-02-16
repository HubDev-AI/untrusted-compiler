# M38-S44 Outbound HTTP Request-Parser Fallback Diagnostics Hardening

## What it is

M38-S44 extends outbound request-parser diagnostics so non-IPv6 parse failures no longer collapse to generic `NET.URL_INVALID`.

It adds deterministic split request-parser diagnostics for scheme/host/port/target parser failures across direct request path and redirect path.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S43 split malformed IPv6 parser diagnostics, but non-IPv6 parser failures still had coarse fallback behavior in key paths.

This slice finishes parser-classification parity for common malformed outbound URL cases.

## How it works

1. Added non-IPv6 parse-status classes to outbound parser:
   - `SEC4_RT_OUTBOUND_URL_PARSE_SCHEME_MISSING`
   - `SEC4_RT_OUTBOUND_URL_PARSE_SCHEME_INVALID`
   - `SEC4_RT_OUTBOUND_URL_PARSE_HOST_INVALID`
   - `SEC4_RT_OUTBOUND_URL_PARSE_PORT_INVALID`
   - `SEC4_RT_OUTBOUND_URL_PARSE_TARGET_INVALID`
2. Added direct request-path deterministic diagnostics:
   - `NET.REQUEST_SCHEME_MISSING`
   - `NET.REQUEST_SCHEME_INVALID`
   - `NET.REQUEST_HOST_INVALID`
   - `NET.REQUEST_PORT_INVALID`
   - `NET.REQUEST_TARGET_INVALID`
3. Extended redirect resolver parse-status mapping so malformed redirect targets keep parser-class diagnostics instead of generic redirect-invalid collapse.
4. Extended redirect-path mapping in `sec4_rt_outbound_http_get_handle(...)` for new non-IPv6 parser-class diagnostics.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Parser and resolver status enums are larger and more explicit.
- Failure mapping code is more verbose, but diagnostics are now precise and deterministic for operator/debug workflows.

## Next

1. Add structured parser diagnostics details (`phase`, `component`) to `NET.REQUEST_*` responses.
2. Add harness assertions for deterministic `details[]` payload values on parser errors.
